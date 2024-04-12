use <utils.scad>

module keycap(choc=false) {
    translate([0,0,choc?4:5]) hull() {
        linear_extrude(0.1) square([choc?17.5:18.5, choc?16.5:18.5], center=true);
        translate([0, 0, choc?3:7.5]) linear_extrude(0.1) rounded_square(12, r=1.5, center=true);
    }
}

module switch(choc=false) {
    translate([0,0,choc?-2.2:-5]) linear_extrude(choc?3:6) rounded_square([13.5,13.5], r=1, center=true);
    linear_extrude(choc?3:6, scale=0.8) rounded_square([15, 15], r=1, center=true);
    translate([0,0,choc?-5:-8]) cylinder(h=5, d=4);
}

keycap();
