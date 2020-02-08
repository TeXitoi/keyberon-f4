use <utils.scad>
use <usb-c-pill.scad>
use <key.scad>
include <printing.scad>

switch_hole=14.1;// by spec should be 14, can be adjusted for printer imprecision
inter_switch=19.05;
thickness=1.6;
d=2.54;
deltas=[-d,0,d,0,-3*d,-4*d];
nb_cols=6;
nb_rows=4;
nb_thumbs=4;
hand_spacing=20;
hand_angle=30;
top=19.92;// should be calculated...

// insert hole, can be adjusted depending on the size of your insert
// or if you use autotaping screws
insert_diameter=3.2;
insert_height=4.6;

module key_placement() {
     for (s=[-1,1]) {
          translate([s * hand_spacing/2,0,0]) rotate([0,0,s*hand_angle/2]) {
               for (j=[0:nb_cols-1]) {
                    translate([s*(j+0.5)*inter_switch, deltas[j], 0]) {
                         for (i=[0.5:nb_rows]) {
                              translate([0, -i*inter_switch, 0]) children();
                         }
                    }
               }
               translate([0, -(0.5+nb_rows)*inter_switch + deltas[0]])
                    translate([s*inter_switch/2,-inter_switch/2,0])
                    rotate([0,0,s*26.5])
                    translate([-s*inter_switch/2,inter_switch/2,0])
                    children();
               for (j=[0:nb_thumbs-2]) {
                    translate([s*(j+1)*inter_switch, -(0.5+nb_rows)*inter_switch + min(deltas[j], deltas[j+1])]) children();
               }
          }
     }
}

module screw_placement() {
     pill_placement() {
          for (i=[-1,1]) {
               translate([i*20,-6,-pill_depth]) children();
          }
          translate([0,-61,-pill_depth]) children();
     }
     for (s=[-1,1]) {
          translate([s * hand_spacing/2,0,0]) rotate([0,0,s*hand_angle/2]) {
               offset=3.5;
               translate([s*inter_switch*0.5-s*offset,-inter_switch*nb_rows-offset+deltas[0],0]) children();
               translate([s*inter_switch*3.75,-inter_switch*(nb_rows+0.25)+deltas[3],0]) children();
               translate([s*(inter_switch*4+offset),deltas[4]+offset,0]) rotate([0,0,-s*hand_angle/2]) children();
          }
     }
}

pill_depth=-0.6-2-1.6;
module pill_placement() {
     translate([0,top,pill_depth]) children();
}

module pill_cube(epsilon=0) {
     pill_placement() {
          translate([-25/2-epsilon,-55-epsilon,0]) cube([25+2*epsilon, 1.2+55+2*epsilon,-pill_depth-0.2]);
     }
}

module plate() {
     difference() {
          union() {
               hull() {
                    key_placement() translate([0,0,-thickness]) linear_extrude(thickness)
                         rounded_square([22, 22], r=2, center=true);
               }
               pill_cube();
          }
          key_placement() cube([switch_hole, switch_hole, 3*thickness], center=true);
          pill_placement() usb_c_pill_pocket();
          screw_placement() translate([0,0,-2]) {
               cylinder(d=4, h=thickness*3, center=true);
          }
     }
}

module case() {
     difference() {
          union() {
               difference() {
                    hull() {
                         key_placement() translate([0,0,-9]) linear_extrude(9-thickness)
                              rounded_square([22, 22], r=2, center=true);
                    }
                    hull() key_placement() translate([0,0,-8]) linear_extrude(8)
                         square([switch_hole+0.5, switch_hole+0.5], center=true);
                    //pill_placement() usb_c_pill_pocket();
                    pill_cube(epsilon=0.2);
               }
               pill_placement() translate([0,-0.1-45/2,-pill_depth-9+(9+pill_depth)/2])
                    cube([10,45,9+pill_depth], center=true);
               screw_placement() {
                    translate([0,0,-8]) cylinder(d=8.8, h=8-thickness);
               }
          }
          // screw holes
          screw_placement() {
               translate([0, 0, -thickness]) cylinder(d=4, h=4*2, center=true);
          }
     }
}

color([0.3,0.3,0.3]) {
     //intersection() {
          //cube([75,100,30], center=true);
          plate();
          case();
     //}
}

color([1,1,1,0.8]) key_placement() switch();
color([0.9,0.9,0.9]) key_placement() keycap();
color([0.7,0.7,0.7]) screw_placement() {
     cylinder(d=5.3, h=1.3);
     translate([0,0,-6]) cylinder(d=2.7, h=6);
}
pill_placement() usb_c_pill();
