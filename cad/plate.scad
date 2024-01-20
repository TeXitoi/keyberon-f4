use <utils.scad>
use <usb-c-pill.scad>
use <key.scad>
include <printing.scad>

switch_hole=14;// by spec should be 14, can be adjusted for printer imprecision
inter_switch=19.05;
thickness=1.6;// plate thinkness
d=2.54;
deltas=[-d,0,d,0,-4*d,-5*d];// column stagger
nb_cols=6;
nb_rows=3;
nb_thumbs=4;
hand_spacing=20;
hand_angle=30;
center_screw=false;
case_border=5;
switch_depth=8;// 8 for MX, 5 for choc
top=9;// might need some tweeks if you cange hand_angle or first delta
top_wide=43;

// insert hole, can be adjusted depending on the size of your insert
// or if you use autotaping screws
insert_diameter=3.2;
insert_height=4.6;

module one_side_key_placement(side, nb_c, nb_r, nb_t) {
     translate([side * hand_spacing/2,0,0]) rotate([0,0,side * hand_angle/2]) {
          for (j=[0:nb_c-1]) {
               translate([side*(j+0.5)*inter_switch, deltas[j], 0]) {
                    for (i=[0.5:nb_r]) {
                         translate([0, -i*inter_switch, 0]) children();
                    }
               }
          }
          translate([0, -(0.5+nb_r)*inter_switch + deltas[0]])
               translate([side*inter_switch/2,-inter_switch/2,0])
               rotate([0,0,side*26.5])
               translate([-side*inter_switch/2,inter_switch/2,0])
               children();
          for (j=[0:nb_t-2]) {
               translate([side*(j+1)*inter_switch, -(0.5+nb_r)*inter_switch + min(deltas[j], deltas[j+1])]) children();
          }
     }
}

module key_placement() {
     for (side=[-1,1]) {
          one_side_key_placement(side, nb_cols, nb_rows, nb_thumbs) children();
     }
}

module outline(border, r) {
     base=15;
     union() {
          for (side=[-1,1]) {
               hull() one_side_key_placement(side, nb_cols, nb_rows, nb_thumbs)
                    rounded_square([base+border*2, base+border*2], r=r, center=true);
          }
          hull() for (side=[-1,1]) {
               one_side_key_placement(side, nb_c=1, nb_r=nb_rows, nb_t=nb_thumbs)
                    rounded_square([base+border*2, base+border*2], r=r, center=true);
          }
          translate([0, -20+top-case_border+border+1.2]) rounded_square([top_wide+border*2, 40], center=true, r=r);
     }
}

module screw_placement() {
     pill_placement() {
          for (i=[-1,1]) {
               translate([i*18,-5,-pill_depth]) children();
          }
          if (center_screw) translate([0,-61,-pill_depth]) children();
     }
     for (s=[-1,1]) {
          translate([s * hand_spacing/2,0,0]) rotate([0,0,s*hand_angle/2]) {
               offset=3.5;
               translate([s*inter_switch*0.5-s*offset,-inter_switch*nb_rows-offset+deltas[0],0]) children();
               if (nb_cols >= 5) {
                    translate([s*inter_switch*3.75,-inter_switch*(nb_rows+0.25)+deltas[3],0]) children();
                    translate([s*(inter_switch*4+offset),deltas[4]+offset,0]) rotate([0,0,-s*hand_angle/2]) children();
               }
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
              difference() {
                  translate([0,0,-switch_depth]) linear_extrude(switch_depth) outline(case_border, r=2);
                  translate([0, 0, -5-switch_depth]) linear_extrude(switch_depth) {
                      outline(0, r=0);
                  }
              }
              pill_cube();
          }
          key_placement() {
              cube([switch_hole, switch_hole, 3*switch_depth], center=true);
              translate([0, 0, -thickness-switch_depth])
                  cube([switch_hole+1, switch_hole+1, 2*switch_depth], center=true);
          }
          pill_placement() usb_c_pill_pocket();
          screw_placement() translate([0,0,-thickness-switch_depth]) {
               cylinder(d=4, h=switch_depth*3, center=true);
               cylinder(d=10, h=switch_depth);
          }
     }
}

module case() {
     difference() {
          union() {
               case_thichness=1.6;
               case_depth=switch_depth+case_thichness;
               difference() {
                    translate([0,0,-case_depth]) linear_extrude(case_thichness) outline(case_border, r=2);
                    //pill_placement() usb_c_pill_pocket();
                    pill_cube(epsilon=0.2);
               }
               pill_placement() translate([0,-0.1-45/2,-pill_depth-case_depth+(case_depth+pill_depth)/2])
                    cube([10,45,case_depth+pill_depth-0.4], center=true);
               screw_placement() {
                    translate([0,0,-switch_depth]) cylinder(d=8.8, h=switch_depth-thickness);
               }
          }
          // screw holes
          screw_placement() {
               translate([0, 0, -thickness]) cylinder(d=4, h=4*2, center=true);
          }
     }
}

color([0.3,0.3,0.3]) plate();
color([0.4,0.4,0.4]) case();


color([1,1,1,0.8]) key_placement() switch();
color([0.9,0.9,0.9]) key_placement() keycap();
color([0.7,0.7,0.7]) screw_placement() {
     cylinder(d=5.3, h=1.3);
     translate([0,0,-4]) cylinder(d=2.7, h=4);
}
pill_placement() usb_c_pill();
