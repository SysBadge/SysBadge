
generated:
	mkdir -p $@

.PHONY: generated/sysbadge.h
generated/sysbadge.h: generated
	cbindgen ../.. > $@
