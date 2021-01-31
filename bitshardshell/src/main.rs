use image::io::Reader as ImageReader;
use ncurses::*;

fn scaled_pixel(x: i32, y: i32, val: u32, scaleX: i32, scaleY: i32) {
    for w in 0..scaleX {
        for h in 0..scaleY {
            mvaddch(y * scaleY + h as i32, x * scaleX + w as i32, val);
        }
    }
}

fn main() {
    initscr();
    raw();

    keypad(stdscr(), true); // Allow F1 etc

    noecho();

    let img = ImageReader::open("testimg.png")
        .expect("Couldn't open 'testimg.png'")
        .decode()
        .expect("Couldn't decode image");

    let rgb = img.as_rgb8().expect("Couldn't read image as rgb8");

    for (x, y, pixel) in rgb.enumerate_pixels() {
        let val = match pixel[0] {
            255 => '#' as u32,
            0 => ' ' as u32,
            _ => '?' as u32,
        };
        scaled_pixel(x as i32, y as i32, val, 4, 3);
    }

    refresh();
    getch();
    endwin();
}
