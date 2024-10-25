use <utils.scad>
use <usb-c-pill.scad>
use <key.scad>
include <printing.scad>

nb_cols=5;
nb_rows=3;
nb_thumbs=3;
choc=true;

inter_switch_x=choc?18:19.05;
inter_switch_y=choc?17:19.05;
d=inter_switch_y/8;
deltas=[0,2,6,2,-10];// column stagger

rotate_thumb_switch=false;
aligned_thumbs=false;
center_screw=true;
center_screw_offset=(nb_rows+0.75)*inter_switch_y;

hand_spacing=22;
hand_angle=30;
top=12;// might need some tweeks if you cange hand_angle or first delta

switch_hole=choc?13.8:14;// can be adjusted for printer imprecision
thickness=1.6;// plate thinkness
case_border=3;
rounding=2;
switch_depth=choc?5:8;// 8 for MX, 5 for choc
top_wide=51;

// insert hole, can be adjusted depending on the size of your insert
// or if you use autotaping screws
insert_diameter=4;

module one_side_key_placement(side, nb_c, nb_r, nb_t) {
     translate([side * hand_spacing/2,0,0]) rotate([0,0,side * hand_angle/2]) {
          for (j=[0:nb_c-1]) {
               translate([side*(j+0.5)*inter_switch_x, deltas[j], 0]) {
                    for (i=[0.5:nb_r]) {
                         translate([0, -i*inter_switch_y, 0]) children();
                    }
               }
          }
          min_deltas=min([for (i=[0:nb_t-1]) deltas[i]]);
          translate([0, -(0.5+nb_r)*inter_switch_y + (aligned_thumbs?min_deltas:deltas[0])])
               translate([side*inter_switch_x/2,-inter_switch_y/2,0])
               rotate([0,0,(rotate_thumb_switch?side:0)*26.5])
               translate([-side*inter_switch_x/2,inter_switch_y/2,0])
               children();
          for (j=[0:nb_t-2]) {
              delta=aligned_thumbs?min_deltas:min(deltas[j], deltas[j+1]);
              translate([side*(j+1)*inter_switch_x, -(0.5+nb_r)*inter_switch_y + delta]) children();
          }
     }
}

module key_placement() {
     for (side=[-1,1]) {
          one_side_key_placement(side, nb_cols, nb_rows, nb_thumbs) children();
     }
}

module outline(border, r, base=choc?[18, 17]:[19.05, 19.05], no_top=false) {
    union() {
        for (side=[-1,1]) {
            hull() one_side_key_placement(side, nb_cols, nb_rows, nb_thumbs)
                rounded_square([base[0]+border*2, base[1]+border*2], r=r, center=true);
        }
        hull() for (side=[-1,1]) {
            one_side_key_placement(side, nb_c=1, nb_r=nb_rows, nb_t=nb_thumbs)
                rounded_square([base[0]+border*2, base[1]+border*2], r=r, center=true);
        }
        if (!no_top)
            translate([0, -20+top+1.2])
                rounded_square([top_wide+border*2, 40], center=true, r=r);
    }
}

module screw_placement() {
     pill_placement() {
          for (i=[-1,1]) {
               translate([i*18,-5,-pill_depth]) children();
          }
     }
     if (center_screw) translate([0,-center_screw_offset,0]) children();
     for (s=[-1,1]) {
          translate([s * hand_spacing/2,0,0]) rotate([0,0,s*hand_angle/2]) {
               offset=3.5;
               if (rotate_thumb_switch) {
                  translate([s*inter_switch_x*0.5-s*offset,-inter_switch_y*nb_rows-offset+deltas[0],0]) children();
               }
               if (nb_cols >= 5) {
                    translate([s*inter_switch_x*3.75,-inter_switch_y*(nb_rows+0.25)+deltas[3],0]) children();
                    translate([s*(inter_switch_x*4+offset),deltas[4]+offset,0]) rotate([0,0,-s*hand_angle/2]) children();
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
                  translate([0,0,-switch_depth]) linear_extrude(switch_depth) outline(case_border, r=rounding);
                  translate([0, 0, -(choc?2.2:5)-switch_depth]) linear_extrude(switch_depth) {
                      outline(0, r=0, base=[15,15], no_top=true);
                  }
              }
              pill_cube();
          }
          key_placement() {
              cube([switch_hole, switch_hole, 3*switch_depth], center=true);
              translate([0, 0, -thickness-switch_depth])
                  cube([15, 15, 2*switch_depth], center=true);
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
                    translate([0,0,-case_depth]) linear_extrude(case_thichness) outline(case_border, r=rounding);
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
               translate([0, 0, -thickness]) cylinder(d=insert_diameter, h=4*2, center=true);
          }
     }
}

color([0.3,0.3,0.3]) plate();
color([0.4,0.4,0.4]) case();


color([1,1,1,0.8]) key_placement() switch(choc=choc);
color([1,1,1]) key_placement() keycap(choc=choc);
color([0.7,0.7,0.7]) screw_placement() {
     cylinder(d=5.3, h=1.3);
     translate([0,0,-4]) cylinder(d=2.7, h=4);
}
pill_placement() usb_c_pill();
