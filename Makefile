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
NATIVE_HEADER := RUSTFLAGS="-C target-cpu=native" cargo rustc -r
X86_64_v2_HEADER := RUSTFLAGS="-C target-cpu=x86-64-v2" cargo rustc -r
X86_64_v3_HEADER := RUSTFLAGS="-C target-cpu=x86-64-v3" cargo rustc -r
X86_64_v4_HEADER := RUSTFLAGS="-C target-cpu=x86-64-v4" cargo rustc -r

ifeq ($(OS),Windows_NT)
  ifneq ($(IS_MINGW),1)
    NATIVE_HEADER := cmd /C "set RUSTFLAGS=-C target-cpu=native && cargo rustc -r"
	X86_64_v2_HEADER := cmd /C "set RUSTFLAGS=-C target-cpu=x86-64-v2 && cargo rustc -r"
	X86_64_v3_HEADER := cmd /C "set RUSTFLAGS=-C target-cpu=x86-64-v3 && cargo rustc -r"
	X86_64_v4_HEADER := cmd /C "set RUSTFLAGS=-C target-cpu=x86-64-v4 && cargo rustc -r"
  endif
endif

default:
	$(NATIVE_HEADER) -p terminal --features=dev -- --emit link=$(EXE)$(EXT)

release: create_version_dir
	$(X86_64_v2_HEADER) -p terminal -- --emit link=$(X86_64_V2)$(EXT)
	$(X86_64_v3_HEADER) -p terminal -- --emit link=$(X86_64_V3)$(EXT)
	$(X86_64_v4_HEADER) -p terminal -- --emit link=$(X86_64_V4)$(EXT)

value_gen:
	$(NATIVE_HEADER) -p datagen --features=value_datagen -- --emit link=datagen$(EXT)

policy_gen:
	$(NATIVE_HEADER) -p datagen --features=policy_datagen -- --emit link=datagen$(EXT)

policy_trainer:
	$(NATIVE_HEADER) -p trainer --features=policy_trainer -- --emit link=trainer$(EXT)

policy_util:
	$(NATIVE_HEADER) -p trainer --features=policy_interleave -- --emit link=policy_interleave$(EXT)

value_trainer:
	$(NATIVE_HEADER) -p trainer --features=value_trainer -- --emit link=trainer$(EXT)

ifneq ("$(wildcard $(RELEASE_DIR))","")
create_version_dir:
else
create_version_dir:
	mkdir -p $(RELEASE_DIR)
endif