FORT = gfortran
F_DIR = $(OUT_DIR)/srcf
FFLAGS = -O3 -fimplicit-none -arch arm64 -mmacosx-version-min=111.0 -J$(F_DIR) -I$(F_DIR) -L$(F_DIR)

OBJECTS= $(F_DIR)/foo.o

OBJECTS_R= foo.o

all: $(OBJECTS_R)
	$(AR) rvu $(F_DIR)/fenda.a $(OBJECTS)

clean:
	rm -f *.o
	rm -f *.mod
	rm -f fenda.a

.SUFFIXES: .o .f .f90

.f90.o:
	$(FORT) $(FFLAGS) -c $< $(F_DIR)/$@

.f.o:
	$(FORT) $(FFLAGS) -c $< $(F_DIR)/$@
