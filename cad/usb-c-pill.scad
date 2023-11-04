use <utils.scad>

height=52.9;
width=20.8;
thickness=1.6;
cin=2.54;
e=0.1;
button_height=2+thickness;

module buttons() {
     for (coord=[[2.4,-15,0], [-2.4,-15,0], [-2,-41,0]]) {
          translate(coord) translate([0,-e,button_height]) {
               children();
          }
     }
}

module usb_c_connector(epsilon=0) {
     translate([0,-e+epsilon,thickness+3.2/2]) rotate([90,0,0]) linear_extrude(7.5+2*epsilon) {
          rounded_square([9+2*epsilon, 3.2+2*epsilon], r=1+epsilon, center=true);
     }
}

module usb_c_pill() {
     translate([0,-e,0]) {     
          translate([0,-height/2,0]) {
               // PCB
               color([0.2,0.2,0.2]) linear_extrude(thickness)
                    square([width, height], center=true);
               // MCU
               color([0.1,0.1,0.1]) translate([0,0,1.2]) rotate([0,0,45])
                    linear_extrude(0.7) square([6.9,6.9], center=true);
          }

          color([0.8,0.8,0.8]) {
               // pins
               for (i=[-1, 1]) {
                    for (j=[0:19]) {
                         translate([3*i*cin,-cin/2-j*cin,-0.05]) cylinder(d=1.6, h=thickness+0.1);
                    }
               }
          }
     }
     color([0.8,0.8,0.8]) usb_c_connector();
     // buttons
     buttons() {
          color([0.1,0.1,0.1]) linear_extrude(0.6) rounded_square([2, 2.9], center=true, r=0.99);
          color([0.8,0.8,0.8]) translate([0,0,-button_height+1]) linear_extrude(button_height-1) square([3.3,4.2], center=true);
     }
}

module usb_c_pill_pocket() {
     translate([0,-height/2-e,-e]) {
         translate([0, 0, -10]) linear_extrude(e+thickness+10) square([width+2*e, height+2*e], center=true);
          linear_extrude(e+button_height) square([width-2, height+2*e], center=true);
     }
     buttons() {
          translate([0, 0, -0.1]) linear_extrude(3, scale=3) rounded_square([2+0.7, 2.9+0.7], center=true, r=2.69/2);
     }
     usb_c_connector(epsilon=e);
     translate([0,1.6,0]) usb_c_connector(epsilon=e);
}

usb_c_pill();
color([0.5, 0.5, 0.5, 0.3]) usb_c_pill_pocket();
