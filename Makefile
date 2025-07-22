EXE = jackal
VER = X.X.X

RELEASE_DIR = releases/$(VER)

# Define binary path
X86_64_V2 := $(RELEASE_DIR)/$(EXE)-$(VER)-x86-64-v2
X86_64_V3 := $(RELEASE_DIR)/$(EXE)-$(VER)-x86-64-v3
X86_64_V4 := $(RELEASE_DIR)/$(EXE)-$(VER)-x86-64-v4

# Make sure to end binaries with .exe on windows
ifeq ($(OS),Windows_NT)
	EXT := .exe
else 
	EXT = 
endif

# Define correct RUSTFLAGS header
NATIVE_HEADER := RUSTFLAGS="-Ctarget-cpu=native" cargo rustc -r
X86_64_v2_HEADER := RUSTFLAGS="-Ctarget-cpu=x86-64-v2" cargo rustc -r
X86_64_v3_HEADER := RUSTFLAGS="-Ctarget-cpu=x86-64-v3" cargo rustc -r
X86_64_v4_HEADER := RUSTFLAGS="-Ctarget-cpu=x86-64-v4" cargo rustc -r

ifeq ($(OS),Windows_NT)
  ifneq ($(IS_MINGW),1)
    NATIVE_HEADER := cmd /C "set RUSTFLAGS=-Ctarget-cpu=native && cargo rustc -r"
	X86_64_v2_HEADER := cmd /C "set RUSTFLAGS=-Ctarget-cpu=x86-64-v2 && cargo rustc -r"
	X86_64_v3_HEADER := cmd /C "set RUSTFLAGS=-Ctarget-cpu=x86-64-v3 && cargo rustc -r"
	X86_64_v4_HEADER := cmd /C "set RUSTFLAGS=-Ctarget-cpu=x86-64-v4 && cargo rustc -r"
  endif
endif

default:
	$(NATIVE_HEADER) --bin jackal --features=pext -- --emit link=$(EXE)$(EXT)

release: create_version_dir
	$(X86_64_v2_HEADER) -p terminal -- --emit link=$(X86_64_V2)$(EXT)
	$(X86_64_v3_HEADER) -p terminal -- --emit link=$(X86_64_V3)$(EXT)
	$(X86_64_v4_HEADER) -p terminal -- --emit link=$(X86_64_V4)$(EXT)

gen:
	$(NATIVE_HEADER) -p datagen --features=datagen -- --emit link=datagen$(EXT)

trainer:
	$(NATIVE_HEADER) -p trainer -- --emit link=trainer$(EXT)

ifneq ("$(wildcard $(RELEASE_DIR))","")
create_version_dir:
else
create_version_dir:
	mkdir -p $(RELEASE_DIR)
endif