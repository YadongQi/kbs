AS_TYPE ?= coco-as
HTTPS_CRYPTO ?= rustls
POLICY_ENGINE ?=

COCO_AS_INTEGRATION_TYPE ?= builtin

INSTALL_DESTDIR ?= /usr/local/bin

ifeq ($(AS_TYPE), coco-as)
  AS_FEATURE = $(AS_TYPE)-$(COCO_AS_INTEGRATION_TYPE)
else
  AS_FEATURE = $(AS_TYPE)
endif

build: background-check-kbs

.PHONY: background-check-kbs
background-check-kbs:
	cargo build -p kbs --locked --release --no-default-features --features $(AS_FEATURE),resource,$(HTTPS_CRYPTO),$(POLICY_ENGINE)

.PHONY: passport-issuer-kbs
passport-issuer-kbs:
	cargo build -p kbs --locked --release --no-default-features --features $(AS_FEATURE),$(HTTPS_CRYPTO)
	mv ../target/release/kbs ../target/release/issuer-kbs

.PHONY: passport-resource-kbs
passport-resource-kbs:
	cargo build -p kbs --locked --release --no-default-features --features $(HTTPS_CRYPTO),resource,$(POLICY_ENGINE)
	mv ../target/release/kbs ../target/release/resource-kbs

install-kbs:
	install -D -m0755 ../target/release/kbs $(INSTALL_DESTDIR)
	install -D -m0755 ../target/release/kbs-client $(INSTALL_DESTDIR)

install-issuer-kbs:
	install -D -m0755 ../target/release/issuer-kbs $(INSTALL_DESTDIR)
	install -D -m0755 ../target/release/kbs-client $(INSTALL_DESTDIR)

install-resource-kbs:
	install -D -m0755 ../target/release/resource-kbs $(INSTALL_DESTDIR)
	install -D -m0755 ../target/release/kbs-client $(INSTALL_DESTDIR)

uninstall:
	rm -rf $(INSTALL_DESTDIR)/kbs $(INSTALL_DESTDIR)/kbs-client $(INSTALL_DESTDIR)/issuer-kbs $(INSTALL_DESTDIR)/resource-kbs

check:
	cargo test -p kbs -p api-server -p kbs-client

lint:
	cargo clippy -p kbs -p api-server -p kbs-client -- -D warnings -A clippy::enum_variant_names

format:
	cargo fmt -p kbs -p api-server -p kbs-client -- --check --config format_code_in_doc_comments=true

clean:
	cargo clean

