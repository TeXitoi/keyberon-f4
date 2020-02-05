all: case.stl plate.stl

%.stl: %.printing.scad
	openscad -o $@ $<

case.stl: case.printing.scad plate.scad printing.scad utils.scad usb-c-pill.scad
plate.stl: case.printing.scad plate.scad printing.scad utils.scad usb-c-pill.scad
