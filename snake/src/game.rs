use piston_window::types::Color;
use piston_window::*;

use rand::{thread_rng, Rng};

use crate::draw::{draw_block, draw_rectangle, draw_rotated_rectangle};
use crate::snake::{Direction, Snake};

// colors in RGB-O
const FOOD_COLOR: Color = [0.80, 0.0, 0.0, 1.0];
const BORDER_COLOR: Color = [0.40, 0.30, 0.0, 1.0];
const GAMEOVER_COLOR: Color = [0.90, 0.0, 0.0, 0.5];
const OBSTACLE_COLOR: Color = [0.0, 0.3, 0.1, 1.0];

const MOVING_PERIOD: f64 = 0.1;
const RESTART_TIME: f64 = 1.0;

pub struct Game {
    snake: Snake,
    obstacles: Vec<Obstacle>,

    food_exists: bool,
    food_x: i32,
    food_y: i32,

    width: i32,
    height: i32,

    game_over: bool,
    waiting_time: f64,
}

pub struct Obstacle {
    x: i32,
    y: i32,
}

impl Game {
    pub fn make_new(width: i32, height: i32) -> Game {
        let mut game = Game {
            snake: Snake::make_new(2, 2),
            obstacles: Vec::new(),
            waiting_time: 0.0,
            food_exists: true,
            food_x: 10,
            food_y: 12,
            width,
            height,
            game_over: false,
        };
        game.add_obstacles();
        game
    }

    pub fn add_obstacles(&mut self) {
        self.obstacles.clear();

        let mut rng = thread_rng();

        for _ in 0..4 {
            let mut x = rng.gen_range(2..self.width - 2);
            let mut y = rng.gen_range(2..self.height - 2);

            while self.snake.overlap_tail(x, y) || self.check_overlap_with_food(x, y) {
                x = rng.gen_range(2..self.width - 2);
                y = rng.gen_range(2..self.height - 2);
            }

            self.obstacles.push(Obstacle { x, y });
        }
    }


    pub fn key_pressed(&mut self, key: Key) {
        if self.game_over {
            return;
        }

        let dir = match key {
            Key::Up => Some(Direction::Up),
            Key::Down => Some(Direction::Down),
            Key::Left => Some(Direction::Left),
            Key::Right => Some(Direction::Right),
            _ => Some(self.snake.head_direction()),
        };

        if let Some(dir) = dir {
            if dir == self.snake.head_direction().opposite() {
                return;
            }
        }

        self.update_snake(dir);
    }

    pub fn draw(&self, con: &Context, g: &mut G2d) {
        self.snake.draw(con, g);

        if self.food_exists {
            draw_block(FOOD_COLOR, self.food_x, self.food_y, con, g);
        }

        for obs in &self.obstacles {
            draw_block(OBSTACLE_COLOR, obs.x, obs.y, con, g);
            draw_block(OBSTACLE_COLOR, obs.x, obs.y - 2, con, g);
            draw_block(OBSTACLE_COLOR, obs.x, obs.y - 1, con, g);
            draw_block(OBSTACLE_COLOR, obs.x, obs.y + 1, con, g);
            draw_block(OBSTACLE_COLOR, obs.x, obs.y + 2, con, g);

            draw_rotated_rectangle(OBSTACLE_COLOR, obs.x - 1, obs.y - 1, 2.0, 0.4, true, con, g);
            draw_rotated_rectangle(
                OBSTACLE_COLOR,
                obs.x + 1,
                obs.y + 1,
                2.0,
                0.4,
                false,
                con,
                g,
            );
        }

        draw_rectangle(BORDER_COLOR, 0, 0, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, self.height - 1, self.width, 1, con, g);
        draw_rectangle(BORDER_COLOR, 0, 0, 1, self.height, con, g);
        draw_rectangle(BORDER_COLOR, self.width - 1, 0, 1, self.height, con, g);

        if self.game_over {
            draw_rectangle(GAMEOVER_COLOR, 0, 0, self.width, self.height, con, g);
        }
    }

    fn check_overlap_with_food(&self, x: i32, y: i32) -> bool {
        let food_positions = vec![
            (self.food_x, self.food_y),  
            (self.food_x - 1, self.food_y - 1),  
            (self.food_x + 1, self.food_y + 1)
        ];
    
        let obstacle_positions = vec![
            (x, y),            
            (x, y + 1), (x, y - 1),      
            (x, y + 2), (x, y - 2),
            (x - 1, y - 1), (x + 1, y + 1)     
        ];
    
        for (ox, oy) in obstacle_positions {
            if food_positions.contains(&(ox, oy)) {
                return true;
            }
        }
    
        false
    }

    pub fn update(&mut self, delta_time: f64) {
        self.waiting_time += delta_time;

        if self.game_over {
            if self.waiting_time > RESTART_TIME {
                self.restart();
                self.add_obstacles();
            }
            return;
        }

        if !self.food_exists {
            self.add_food();
        }


        if self.waiting_time > MOVING_PERIOD {
            self.update_snake(None);
        }
    }

    fn check_eating(&mut self) {
        let (head_x, head_y): (i32, i32) = self.snake.head_position();
        if self.food_exists && self.food_x == head_x && self.food_y == head_y {
            self.food_exists = false;
            self.snake.restore_tail();
        }
    }

    fn check_if_snake_alive(&self, dir: Option<Direction>) -> bool {
        let (next_x, next_y) = self.snake.next_head(dir);

        if self.snake.overlap_tail(next_x, next_y) {
            return false;
        }

        for obs in &self.obstacles {
            if (next_x == obs.x && next_y == obs.y) || (next_x == obs.x && next_y == obs.y - 1)
                || (next_x == obs.x && next_y == obs.y - 2) || (next_x == obs.x && next_y == obs.y + 2)
                || (next_x == obs.x && next_y == obs.y + 1){
                return false;
            }
            let left_arm_x = obs.x - 1;
            let left_arm_y = obs.y - 1;
            let right_arm_x = obs.x + 1;
            let right_arm_y = obs.y + 1;

            // Define bounding boxes around the rotated arms
            if (next_x >= left_arm_x && next_x <= left_arm_x + 1 && next_y >= left_arm_y && next_y <= left_arm_y + 1)
                || (next_x >= right_arm_x - 1 && next_x <= right_arm_x && next_y >= right_arm_y - 1 && next_y <= right_arm_y){
                return false;
            }
        }

        true
    }

    fn add_food(&mut self) {
        let mut rng = thread_rng();

        let mut new_x = rng.gen_range(1..self.width - 1);
        let mut new_y = rng.gen_range(1..self.height - 1);
        while self.snake.overlap_tail(new_x, new_y) {
            new_x = rng.gen_range(1..self.width - 1);
            new_y = rng.gen_range(1..self.height - 1);
        }

        self.food_x = new_x;
        self.food_y = new_y;
        self.food_exists = true;
    }

    fn update_snake(&mut self, dir: Option<Direction>) {
        if self.check_if_snake_alive(dir) {
            self.snake.move_forward(dir, self.width, self.height);
            self.check_eating();
        } else {
            self.game_over = true;
        }
        self.waiting_time = 0.0;
    }

    fn restart(&mut self) {
        self.snake = Snake::make_new(2, 2);
        self.obstacles = vec![];
        self.waiting_time = 0.0;
        self.food_exists = true;
        self.food_x = 6;
        self.food_y = 4;
        self.game_over = false;
    }
}
