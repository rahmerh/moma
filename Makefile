BINARY_NAME = moma
PREFIX ?= /usr/local
INSTALL_BIN_DIR = $(PREFIX)/bin
TARGET = target/release/$(BINARY_NAME)

build:
	@cargo build --release

install:
	@if [ ! -f target/release/$(BINARY_NAME) ]; then \
		echo "Binary not found. Run 'make build' first."; \
		exit 1; \
	fi
	@install -Dm755 $(TARGET) $(INSTALL_BIN_DIR)/$(BINARY_NAME)
	@echo "Installed moma to $(INSTALL_BIN_DIR)/$(BINARY_NAME)."

uninstall:
	rm -f $(INSTALL_BIN_DIR)/$(BINARY_NAME)
