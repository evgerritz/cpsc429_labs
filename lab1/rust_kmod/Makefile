# SPDX-License-Identifier: GPL-2.0
RUST_LIB_SRC ?= $(rustc_sysroot)/lib/rustlib/src/rust/library
core-cfgs = \
    --cfg no_fp_fmt_parse

obj-$(CONFIG_SAMPLE_RUST_MINIMAL)		+= rust_minimal.o
obj-$(CONFIG_SAMPLE_RUST_PRINT)			+= rust_print.o
obj-$(CONFIG_SAMPLE_RUST_MODULE_PARAMETERS)	+= rust_module_parameters.o
obj-$(CONFIG_SAMPLE_RUST_SYNC)			+= rust_sync.o
obj-$(CONFIG_SAMPLE_RUST_CHRDEV)		+= rust_chrdev.o
obj-$(CONFIG_SAMPLE_RUST_MISCDEV)		+= rust_miscdev.o
obj-$(CONFIG_SAMPLE_RUST_STACK_PROBING)		+= rust_stack_probing.o
obj-$(CONFIG_SAMPLE_RUST_SEMAPHORE)		+= rust_semaphore.o
obj-$(CONFIG_SAMPLE_RUST_SEMAPHORE_C)		+= rust_semaphore_c.o
obj-$(CONFIG_SAMPLE_RUST_RANDOM)		+= rust_random.o
obj-$(CONFIG_SAMPLE_RUST_PLATFORM)		+= rust_platform.o
obj-$(CONFIG_SAMPLE_RUST_NETFILTER)		+= rust_netfilter.o
obj-$(CONFIG_SAMPLE_RUST_ECHO_SERVER)		+= rust_echo_server.o
obj-$(CONFIG_SAMPLE_RUST_FS)			+= rust_fs.o
obj-$(CONFIG_SAMPLE_RUST_SELFTESTS)		+= rust_selftests.o
obj-$(CONFIG_SAMPLE_RUST_MYMEM)		+= rust_mymem.o
obj-$(CONFIG_SAMPLE_MYMEM_TEST)		+= mymem_test.o
obj-$(CONFIG_RUST)		+= exports_mymem_generated.h
obj-$(CONFIG_RUST)		+= exports.o

subdir-$(CONFIG_SAMPLE_RUST_HOSTPROGS)		+= hostprogs



quiet_cmd_exports = EXPORTS $@
      cmd_exports = \
	$(NM) -p --defined-only $< \
		| grep -E ' (T|R|D) ' | cut -d ' ' -f 3 \
		| xargs -Isymbol \
		echo 'EXPORT_SYMBOL_RUST_GPL(symbol);' | grep -v 'module' > $@

$(obj)/exports_mymem_generated.h: $(obj)/rust_mymem.o FORCE
	$(call if_changed,exports)

#quiet_cmd_rustc_library = $(if $(skip_clippy),RUSTC,$(RUSTC_OR_CLIPPY_QUIET)) L $@
#      cmd_rustc_library = \
	OBJTREE=$(abspath $(objtree)) \
	$(if $(skip_clippy),$(RUSTC),$(RUSTC_OR_CLIPPY)) \
		$(filter-out $(skip_flags),$(rust_flags) $(rustc_target_flags)) \
		--emit=dep-info,obj,metadata --crate-type rlib \
		--out-dir $(objtree)/$(obj) -L$(objtree)/$(obj) \
		--crate-name $(patsubst %.o,%,$(notdir $@)) $<; \
	mv $(objtree)/$(obj)/$(patsubst %.o,%,$(notdir $@)).d $(depfile); \
	sed -i '/^\#/d' $(depfile) \
	$(if $(rustc_objcopy),;$(OBJCOPY) $(rustc_objcopy) $@)

#$(obj)/rust_mymem.o: private skip_clippy = 1
#$(obj)/rust_mymem.o: private skip_flags = -Dunreachable_pub
#$(obj)/rust_mymem.o: private rustc_target_flags = $(core-cfgs)
#$(obj)/rust_mymem.o: rust_mymem.rs 
SAMPLE_RUST_MYMEM#	$(call if_changed_dep,rustc_library)
