use crate::system::Member;
use crate::{Button, DrawResult, System};
use alloc::format;
use core::hint::unreachable_unchecked;
use core::ptr;
use embedded_graphics::geometry::AnchorY;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::BinaryColor;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle, StyledDrawable};
use embedded_graphics::text::{Alignment, Text};

#[cfg(not(feature = "invert"))]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::Off;

#[cfg(feature = "invert")]
const BINARY_COLOR_OFF: BinaryColor = BinaryColor::On;

#[cfg(not(feature = "invert"))]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::On;

#[cfg(feature = "invert")]
const BINARY_COLOR_ON: BinaryColor = BinaryColor::Off;

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Select {
    #[default]
    None,
    Select,
    Edit,
}

impl Select {
    pub(crate) fn stroke_with(&self) -> u32 {
        match self {
            Self::None => 1,
            Self::Select => 2,
            Self::Edit => 4,
        }
    }
}

#[derive(Eq, PartialEq, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MemberCell {
    pub id: u16,
}

#[derive(Eq, PartialEq, Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CurrentMembers {
    pub members: [MemberCell; 4],
    pub sel: (u8, Select),
    pub len: u8,
}

impl core::default::Default for CurrentMembers {
    fn default() -> Self {
        Self {
            len: 1,
            sel: (0, Select::Select),
            members: core::default::Default::default(),
        }
    }
}

impl CurrentMembers {
    fn sel_for_cell(&self, idx: u8) -> Select {
        if self.sel.0 == idx {
            self.sel.1
        } else {
            Select::None
        }
    }

    fn button_press(&mut self, button: Button, members: usize) {
        match button {
            Button::Up | Button::Down if self.sel.1 == Select::None => {
                self.sel.1 = Select::Select;
                if button == Button::Up {
                    self.sel.0 = 0;
                } else {
                    self.sel.0 = self.len - 1;
                }
            }
            Button::Up | Button::Down if self.sel.1 == Select::Select => {
                if button == Button::Up {
                    self.sel.0 = dec_wrapping(self.sel.0, self.len - 1);
                } else {
                    self.sel.0 = inc_wrapping(self.sel.0, self.len - 1);
                }
            }
            Button::Up | Button::Down if self.sel.1 == Select::Edit => {
                if button == Button::Up {
                    self.members[self.sel.0 as usize].id =
                        inc_wrapping(self.members[self.sel.0 as usize].id, members as u16 - 1);
                } else {
                    self.members[self.sel.0 as usize].id =
                        dec_wrapping(self.members[self.sel.0 as usize].id, members as u16 - 1);
                }
            }
            Button::C if self.sel.1 == Select::Edit => {
                self.len -= 1;
                if self.sel.0 != (self.len - 1) {
                    self.members[self.sel.0 as usize] = self.members[self.len as usize];
                }
                self.sel.1 = Select::None;
            }
            Button::C => {
                self.sel.1 = Select::None;
                if self.len < 4 {
                    self.len += 1;
                }
            }
            Button::B => {
                if self.sel.1 == Select::Select {
                    self.sel.1 = Select::Edit;
                } else {
                    self.sel.1 = Select::Select;
                }
            }
            _ => {
                #[cfg(feature = "defmt")]
                defmt::warn!("Unhandled member button press: {:?}", button)
            }
        }
    }

    fn draw<D, S>(&self, system: &S, target: &mut D) -> DrawResult<D>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
        S: System,
    {
        debug_assert!(self.len != 0 && self.len <= 4);

        match self.len {
            1 => {
                DrawableMember::new(
                    system.member(self.members[0].id as usize),
                    target.bounding_box(),
                    self.sel_for_cell(0),
                )
                .draw(target)?;
            }
            2 => {
                self.draw_two((0, 1), system, target.bounding_box(), target)?;
            }
            3 => {
                DrawableMember::new(
                    system.member(self.members[0].id as usize),
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Top),
                    self.sel_for_cell(0),
                )
                .draw(target)?;
                DrawableMember::new(
                    system.member(self.members[1].id as usize),
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Center),
                    self.sel_for_cell(1),
                )
                .draw(target)?;
                DrawableMember::new(
                    system.member(self.members[2].id as usize),
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 3, AnchorY::Bottom),
                    self.sel_for_cell(2),
                )
                .draw(target)?;
            }
            4 => {
                self.draw_two(
                    (0, 1),
                    system,
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 2, AnchorY::Top),
                    target,
                )?;
                self.draw_two(
                    (2, 3),
                    system,
                    target
                        .bounding_box()
                        .resized_height(target.bounding_box().size.height / 2, AnchorY::Bottom),
                    target,
                )?;
            }
            _ => unsafe { unreachable_unchecked() },
        }

        Ok(())
    }

    fn draw_two<D, S>(
        &self,
        idx: (u8, u8),
        system: &S,
        bounds: Rectangle,
        target: &mut D,
    ) -> DrawResult<D>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
        S: System,
    {
        DrawableMember::new(
            system.member(self.members[idx.0 as usize].id as usize),
            bounds.resized_height(bounds.size.height / 2, AnchorY::Top),
            self.sel_for_cell(idx.0),
        )
        .draw(target)?;
        DrawableMember::new(
            system.member(self.members[idx.1 as usize].id as usize),
            bounds.resized_height(bounds.size.height / 2, AnchorY::Bottom),
            self.sel_for_cell(idx.1),
        )
        .draw(target)?;

        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug)]
#[repr(u8)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CurrentMenu {
    SystemName,
    Version,
    Member(CurrentMembers),
    InvalidSystem,
}

impl CurrentMenu {
    pub fn change(&mut self, button: Button, members: usize) {
        match self {
            Self::SystemName if button == Button::B => *self = Self::Version,
            Self::Version if button == Button::B => *self = Self::SystemName,
            Self::SystemName if button == Button::C => {
                *self = Self::Member(CurrentMembers::default())
            }
            Self::Member(ref c) if button == Button::C && c.len == 1 && c.sel.1 == Select::Edit => {
                *self = Self::SystemName
            }
            Self::Member(c) => c.button_press(button, members),
            Self::InvalidSystem => (),
            _ => {
                #[cfg(feature = "defmt")]
                defmt::warn!("Unhandled button press: {:?}", button)
            }
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        let ptr = self as *const Self as *const u8;
        unsafe { core::slice::from_raw_parts(ptr, core::mem::size_of::<Self>()) }
    }

    pub fn from_bytes(slice: &[u8]) -> Self {
        let ptr = slice.as_ptr() as *const Self;
        unsafe { ptr::read(ptr) }
    }
}
pub struct Sysbadge<D, S>
where
    D: DrawTarget,
    <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    S: System,
{
    pub display: D,
    pub system: S,
    pub serial: Option<alloc::string::String>,
    current: CurrentMenu,
    hash: u16,
}

impl<D, S> Sysbadge<D, S>
where
    D: DrawTarget,
    <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    S: System,
{
    pub fn new(display: D, system: S) -> Self {
        let current = if system.is_valid() {
            CurrentMenu::SystemName
        } else {
            CurrentMenu::InvalidSystem
        };

        Self {
            display,
            system,
            serial: None,
            current,
            hash: 0,
        }
    }

    pub fn press(&mut self, button: Button) {
        self.current.change(button, self.system.member_count());

        #[cfg(feature = "defmt")]
        defmt::debug!(
            "Pressed button: {:?}, switched to: {:?}",
            button,
            self.current
        );
    }

    pub fn draw(&mut self) -> DrawResult<D, bool> {
        let hash = self.hash();
        if self.hash == hash {
            return Ok(false);
        }

        self.force_draw()?;
        self.hash = hash;
        Ok(true)
    }

    pub fn force_draw(&mut self) -> DrawResult<D> {
        self.display.clear(BINARY_COLOR_OFF.into())?;
        match self.current {
            CurrentMenu::InvalidSystem => self.draw_invalid_system(),
            CurrentMenu::SystemName => self.draw_system_name(),
            CurrentMenu::Version => self.draw_version(),
            CurrentMenu::Member(ref cur) => cur.draw(&self.system, &mut self.display),
        }
    }

    pub fn current(&self) -> &CurrentMenu {
        &self.current
    }

    pub fn set_current(&mut self, state: CurrentMenu) {
        self.current = state
    }

    fn hash(&self) -> u16 {
        let mut crc: crc16::State<crc16::BUYPASS> = crc16::State::new();
        crc.update(unsafe {
            core::slice::from_raw_parts(
                &self.current as *const CurrentMenu as *const u8,
                core::mem::size_of::<CurrentMenu>(),
            )
        });
        crc.get()
    }

    fn draw_system_name(&mut self) -> DrawResult<D> {
        Text::with_alignment(
            self.system.name().as_ref(),
            self.display.bounding_box().center(),
            MonoTextStyle::new(
                &embedded_graphics::mono_font::ascii::FONT_10X20,
                BINARY_COLOR_ON.into(),
            ),
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    fn draw_version(&mut self) -> DrawResult<D> {
        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            BINARY_COLOR_ON.into(),
        );

        Text::with_alignment(
            "Sysbadge",
            self.display.bounding_box().center().x_axis() + Point::new(0, 20),
            text_style,
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        self.draw_version_and_serial(Point::new(5, 60))?;

        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_9X18,
            BINARY_COLOR_ON.into(),
        );

        Text::with_alignment(
            concat!(
                "matrix: ",
                env!("SYSBADGE_MATRIX", "missing matrix configuration")
            ),
            Point::new(5, 105),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;
        Text::with_alignment(
            concat!("web: ", env!("SYSBADGE_WEB", "missing web configuration")),
            Point::new(5, 120),
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        Ok(())
    }

    fn draw_version_and_serial(&mut self, start: Point) -> DrawResult<D> {
        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            BINARY_COLOR_ON.into(),
        );

        let point = Text::with_alignment("Version: ", start, text_style, Alignment::Left)
            .draw(&mut self.display)?;
        Text::with_alignment(
            env!("CARGO_PKG_VERSION"),
            point,
            text_style,
            Alignment::Left,
        )
        .draw(&mut self.display)?;

        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_9X18,
            BINARY_COLOR_ON.into(),
        );

        if let Some(serial) = &self.serial {
            let point = Text::with_alignment(
                "Serial: ",
                start + Point::new(0, 30),
                text_style,
                Alignment::Left,
            )
            .draw(&mut self.display)?;
            Text::with_alignment(serial, point, text_style, Alignment::Left)
                .draw(&mut self.display)?;
        }

        Ok(())
    }

    fn draw_invalid_system(&mut self) -> DrawResult<D> {
        let text_style = MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_10X20,
            BINARY_COLOR_ON.into(),
        );

        Text::with_alignment(
            "System Data Invalid",
            self.display.bounding_box().center().x_axis() + Point::new(0, 40),
            text_style,
            Alignment::Center,
        )
        .draw(&mut self.display)?;

        self.draw_version_and_serial(Point::new(5, 75))?;

        Ok(())
    }

    #[cfg(feature = "simulator")]
    pub fn reset(&mut self) {
        self.current = CurrentMenu::SystemName;
    }
}

fn inc_wrapping<T>(cur: T, max: T) -> T
where
    T: core::ops::Add<T, Output = T>,
    T: Eq,
    T: TryFrom<u8>,
{
    if cur == max {
        unsafe { T::try_from(0).unwrap_unchecked() }
    } else {
        cur + unsafe { T::try_from(1).unwrap_unchecked() }
    }
}

fn dec_wrapping<T>(cur: T, max: T) -> T
where
    T: core::ops::Sub<T, Output = T>,
    T: Eq,
    T: TryFrom<u8>,
{
    if cur == unsafe { T::try_from(0).unwrap_unchecked() } {
        max
    } else {
        cur - unsafe { T::try_from(1).unwrap_unchecked() }
    }
}

pub(crate) struct DrawableMember<C, M>
where
    C: PixelColor + From<BinaryColor>,
    M: Member,
{
    member: M,
    bounds: Rectangle,
    select: Select,
    _color: core::marker::PhantomData<C>,
}

impl<C, M> DrawableMember<C, M>
where
    C: PixelColor + From<BinaryColor>,
    M: Member,
{
    pub fn new(member: M, bounds: Rectangle, select: Select) -> Self {
        Self {
            member,
            bounds,
            select,
            _color: core::marker::PhantomData,
        }
    }

    fn pronoun<D>(&self, target: &mut D) -> DrawResult<D, Point>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let (pos, align, font) = match self.bounds.size.height {
            x if x > 100 => (
                Point::new(5, (self.bounds.size.height - 20) as i32),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_10X20,
            ),
            x if x > 50 => (
                Point::new(5, (self.bounds.size.height - 15) as i32),
                Alignment::Left,
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            x if x > 40 => (
                Point::new((self.bounds.size.width - 5) as i32, 15),
                Alignment::Right,
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            _ => (
                Point::new((self.bounds.size.width - 5) as i32, 15),
                Alignment::Right,
                &embedded_graphics::mono_font::ascii::FONT_6X10,
            ),
        };

        Text::with_alignment(
            &format!("({})", self.member.pronouns().as_ref()),
            self.bounds.top_left + pos,
            MonoTextStyle::new(font, BINARY_COLOR_ON.into()),
            align,
        )
        .draw(target)
    }

    fn name<D>(&self, target: &mut D) -> DrawResult<D, Point>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let (pos, font) = match self.bounds.size.height {
            x if x > 40 => (
                Point::new(5, 25),
                //&embedded_graphics::mono_font::ascii::FONT_10X20,
                &profont::PROFONT_24_POINT,
            ),
            x if x > 20 => (
                Point::new(5, 20),
                &embedded_graphics::mono_font::ascii::FONT_8X13,
            ),
            _ => (
                Point::new(5, 20),
                &embedded_graphics::mono_font::ascii::FONT_6X10,
            ),
        };

        Text::with_alignment(
            self.member.name().as_ref(),
            self.bounds.top_left + pos,
            MonoTextStyle::new(font, BINARY_COLOR_ON.into()),
            Alignment::Left,
        )
        .draw(target)
    }
}

impl<C, M> Drawable for DrawableMember<C, M>
where
    C: PixelColor + From<BinaryColor>,
    M: Member,
{
    type Color = C;
    type Output = Rectangle;

    fn draw<D>(&self, target: &mut D) -> Result<Self::Output, D::Error>
    where
        D: DrawTarget,
        <D as DrawTarget>::Color: From<BinaryColor> + PixelColor,
    {
        let bound_style =
            PrimitiveStyle::with_stroke(BINARY_COLOR_ON.into(), self.select.stroke_with());
        self.bounds.draw_styled(&bound_style, target)?;

        self.name(target)?;
        if !self.member.pronouns().as_ref().is_empty() {
            self.pronoun(target)?;
        }

        Ok(self.bounds)
    }
}
