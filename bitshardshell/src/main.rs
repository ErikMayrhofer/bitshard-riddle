use image::io::Reader as ImageReader;
use image::ImageBuffer;
use image::Rgb;
use ncurses::*;

const O_C: char = '\0';
const CHAR_UNKNOWN: char = '≋';
const CHAR_WALL_V: char = '│';
const CHAR_WALL_H: char = '─';
const CHAR_SOLID: char = '∷';
const TILE_WALL: [[char; 3]; 3] = [
    [CHAR_SOLID, CHAR_SOLID, CHAR_SOLID],
    [CHAR_SOLID, CHAR_SOLID, CHAR_SOLID],
    [CHAR_SOLID, CHAR_SOLID, CHAR_SOLID],
];
const TILE_WALL_O_N: [[char; 3]; 3] = [
    [CHAR_WALL_H, CHAR_WALL_H, CHAR_WALL_H],
    [O_C, O_C, O_C],
    [O_C, O_C, O_C],
];
const TILE_WALL_O_S: [[char; 3]; 3] = [
    [O_C, O_C, O_C],
    [O_C, O_C, O_C],
    [CHAR_WALL_H, CHAR_WALL_H, CHAR_WALL_H],
];
const TILE_WALL_O_E: [[char; 3]; 3] = [
    [O_C, O_C, CHAR_WALL_V],
    [O_C, O_C, CHAR_WALL_V],
    [O_C, O_C, CHAR_WALL_V],
];
const TILE_WALL_O_W: [[char; 3]; 3] = [
    [CHAR_WALL_V, O_C, O_C],
    [CHAR_WALL_V, O_C, O_C],
    [CHAR_WALL_V, O_C, O_C],
];

fn draw_tile<F>(tile: [[char; 3]; 3], render: F)
where
    F: Fn(u32, u32, char) -> (),
{
    for x in 0..3usize {
        for y in 0..3usize {
            if tile[y][x] != O_C {
                render(x as u32, y as u32, tile[y][x]);
            }
        }
    }
}

fn fill_tile<F>(tile: char, render: F)
where
    F: Fn(u32, u32, char) -> (),
{
    for x in 0..3usize {
        for y in 0..3usize {
            render(x as u32, y as u32, tile);
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Air,
    Wall,
    Outside,
}

impl Tile {
    fn from(pixel: &Rgb<u8>) -> Self {
        match (pixel[0], pixel[1], pixel[2]) {
            (255, 0, 0) => Tile::Wall,
            _ => Tile::Air,
        }
    }
}

struct World {
    data: Vec<Tile>,
    width: u32,
    height: u32,
}

//const CORNERS: [[char; 2]; 2] = [['┏', '┓'], ['┗', '┛']];
const CORNERS: [[char; 2]; 2] = [['╭', '╮'], ['╰', '╯']];

impl World {
    fn from(image: &ImageBuffer<Rgb<u8>, Vec<u8>>) -> Self {
        return World {
            data: image.pixels().map(|pixel| Tile::from(pixel)).collect(),
            width: image.width(),
            height: image.height(),
        };
    }
    fn tile_at(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return Tile::Outside;
        }
        return self.data[(y as u32 * self.width + x as u32) as usize];
    }

    fn draw<F>(&self, x: i32, y: i32, render: F)
    where
        F: Fn(u32, u32, char) -> (),
    {
        let tile = self.tile_at(x, y);

        if let Tile::Wall = tile {
            draw_tile(TILE_WALL, &render);
            if let Tile::Air = self.tile_at(x, y - 1) {
                draw_tile(TILE_WALL_O_N, &render);
            }
            if let Tile::Air = self.tile_at(x, y + 1) {
                draw_tile(TILE_WALL_O_S, &render);
            }
            if let Tile::Air = self.tile_at(x + 1, y) {
                draw_tile(TILE_WALL_O_E, &render);
            }
            if let Tile::Air = self.tile_at(x - 1, y) {
                draw_tile(TILE_WALL_O_W, &render);
            }
            for a in 0..2 {
                for b in 0..2 {
                    //Operating on Corner a*2, b*2

                    let x_shoot = a * 2 - 1;
                    let y_shoot = b * 2 - 1;
                    //d
                    let check_x = self.tile_at(x + x_shoot, y) == Tile::Air;
                    let check_xy = self.tile_at(x + x_shoot, y + y_shoot) == Tile::Air;
                    let check_y = self.tile_at(x, y + y_shoot) == Tile::Air;

                    let val = check_x as u8 + check_y as u8 * 2u8 + check_xy as u8 * 4u8;

                    if val == 4 {
                        render(
                            a as u32 * 2,
                            b as u32 * 2,
                            CORNERS[1 - b as usize][1 - a as usize],
                        );
                    }
                    if val == 3 || val == 7 {
                        render(a as u32 * 2, b as u32 * 2, CORNERS[b as usize][a as usize]);
                    }
                }
            }
        } else if let Tile::Air = tile {
            fill_tile(' ', render);
        } else {
            fill_tile(CHAR_UNKNOWN, render);
        }
    }
}

struct Viewport {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Viewport {
    fn render_world(&self, world: &World) {
        let subtile_x = self.x % 3;
        let subtile_y = self.y % 3;

        let mut width = 0;
        let mut height = 0;

        getmaxyx(stdscr(), &mut height, &mut width);

        let originX = width / 2 - self.width * 3 / 2;
        let originY = height / 2 - self.height * 3 / 2;

        for x in 0i32..self.width as i32 + 3 {
            for y in 0i32..self.height as i32 + 3 {
                world.draw(x + self.x / 3, y + self.y / 3, |xoffs, yoffs, ch| {
                    let term_x = x * 3 + xoffs as i32 - subtile_x;
                    let term_y = y * 3 + yoffs as i32 - subtile_y;
                    if term_x > 0 && term_y > 0 {
                        if term_x < self.width * 3 && term_y < self.height * 3 {
                            mvaddstr(
                                originY + term_y as i32,
                                originX + term_x as i32,
                                ch.to_string().as_ref(),
                            );
                        }
                    }
                });
            }
        }

        mvaddstr(self.height / 2 * 3, self.width / 2 * 3, "תּ");
        curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
    }

    fn move_by(&mut self, dx: i32, dy: i32) {
        self.x = self.x + dx;
        self.y = self.y + dy;
    }
}

enum MenuAction {
    Quit,
    CloseMenu,
}

fn menu() -> MenuAction {
    clear();
    addstr("Menu: \n");
    addstr("Q to quit\n");
    addstr("F1 to exit menu\n");

    loop {
        match getch() {
            113 => return MenuAction::Quit,
            KEY_F1 => return MenuAction::CloseMenu,
            _ => {}
        }
    }
}

fn main() {
    setlocale(LcCategory::ctype, "en_GB.UTF-8");

    initscr();
    raw();

    keypad(stdscr(), true); // Allow F1 etc

    noecho();

    let img = ImageReader::open("testimg.png")
        .expect("Couldn't open 'testimg.png'")
        .decode()
        .expect("Couldn't decode image");

    let rgb = img.as_rgb8().expect("Couldn't read image as rgb8");

    let world = World::from(rgb);
    let mut view = Viewport {
        x: 0,
        y: 0,
        width: 20,
        height: 10,
    };

    loop {
        view.render_world(&world);
        refresh();

        let input = getch();
        match input {
            KEY_F1 => match menu() {
                MenuAction::Quit => break,
                MenuAction::CloseMenu => {}
            },
            KEY_LEFT => view.move_by(-1, 0),
            KEY_RIGHT => view.move_by(1, 0),
            KEY_UP => view.move_by(0, -1),
            KEY_DOWN => view.move_by(0, 1),
            _ => {}
        }
    }

    endwin();
}
