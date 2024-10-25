#[cfg(test)]

use crate::img;

#[test]
fn it_main() {
  println!("Start main test");
}

#[test]
fn it_png_save() {
  img::tests::it_img_xport();
}