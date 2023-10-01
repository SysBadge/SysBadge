@0x9cb44429c0cda275;

struct System @0x8c2507c83833987c {
    name @0 :Text;
    members @1 :List(Member);

    union {
        none @2 :Void;
        pkHid @3 :Text;
        pronouns @4 :Text;
    }
}

struct Member @0xf2e2304b05ecec31 {
    name @0 :Text;
    pronouns @1 :Text;
}