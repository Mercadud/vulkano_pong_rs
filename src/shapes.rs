use std::sync::Arc;

use bytemuck::{Pod, Zeroable};
use rand::Rng;
use vulkano::{
    buffer::{BufferUsage, CpuAccessibleBuffer},
    device::Device,
    impl_vertex,
    memory::pool::{PotentialDedicatedAllocation, StdMemoryPoolAlloc},
};

pub fn get_player() -> Vec<Vertex> {
    vec![
        Vertex {
            position: [-0.01, -0.2],
        },
        Vertex {
            position: [-0.01, 0.2],
        },
        Vertex {
            position: [0.01, -0.2],
        },
        Vertex {
            position: [-0.01, 0.2],
        },
        Vertex {
            position: [0.01, -0.2],
        },
        Vertex {
            position: [0.01, 0.2],
        },
    ]
}

pub fn get_ball() -> Vec<Vertex> {
    vec![
        // it's this specific to make is square in 16:9 aspect ratio monitors
        Vertex {
            position: [-0.0084375, -0.015],
        },
        Vertex {
            position: [-0.0084375, 0.015],
        },
        Vertex {
            position: [0.0084375, -0.015],
        },
        Vertex {
            position: [-0.0084375, 0.015],
        },
        Vertex {
            position: [0.0084375, -0.015],
        },
        Vertex {
            position: [0.0084375, 0.015],
        },
    ]
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct Vertex {
    pub(crate) position: [f32; 2],
}

impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Zeroable, Pod)]
pub struct InstanceInfo {
    offset: [f32; 2],
}

impl_vertex!(InstanceInfo, offset);

pub trait Instanceable {
    fn get_instance_buffer(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<[InstanceInfo], PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>;
}

pub struct Player {
    pub position: [f32; 2],
    pub shape: Vec<Vertex>,
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: [-0.95, 0.0],
            shape: get_player(),
        }
    }

    pub fn update_position(&mut self, clicked_up: bool, clicked_down: bool) {
        let mut movement = 0.0;
        if clicked_up {
            movement -= 0.01;
        }
        if clicked_down {
            movement += 0.01;
        }

        let upy = self.shape.get(0).unwrap().position[1] + self.position[1];
        let dpy = self.shape.get(1).unwrap().position[1] + self.position[1];

        if upy <= -1.0 && movement < 0.0 {
            movement = 0.0;
        } else if dpy >= 1.0 && movement > 0.0 {
            movement = 0.0;
        }

        self.position[1] += movement;
    }
}

impl Instanceable for Player {
    fn get_instance_buffer(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<[InstanceInfo], PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>
    {
        let data = vec![InstanceInfo {
            offset: self.position,
        }];

        let e = CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, data).unwrap();
        e
    }
}

pub struct Ball {
    pub position: [f32; 2],
    pub direction: [f32; 2],
    pub shape: Vec<Vertex>,
}

impl Ball {
    pub fn new() -> Self {
        let mut rng = rand::thread_rng();
        let r = rng.gen_range(-0.01..0.01);
        let left_right = rng.gen_bool(0.5);
        Self {
            position: [0.0, 0.0],
            direction: [
                match left_right {
                    true => -0.003,
                    false => 0.003,
                },
                r,
            ],
            shape: get_ball(),
        }
    }

    pub fn update_position(&mut self, player: &Player, opponent: &Opponent) {

        let reflect = |ball : &mut Ball| {
            if ball.direction[0] < 0.0 {
                ball.direction[0] -= 0.002;
            } else {
                ball.direction[0] += 0.002;
            }
            ball.direction[0] = -ball.direction[0];
            if ball.direction[1] < 0.0 {
                ball.direction[1] -= 0.002;
            } else {
                ball.direction[1] += 0.002;
            }
        };


        self.position[0] += self.direction[0];
        self.position[1] += self.direction[1];

        let lbx = self.shape.get(0).unwrap().position[0] + self.position[0];
        let rbx = self.shape.get(2).unwrap().position[0] + self.position[0];
        let uby = self.shape.get(0).unwrap().position[1] + self.position[1];
        let dby = self.shape.get(1).unwrap().position[1] + self.position[1];

        // check collision with player
        {
            let rpx = player.shape.get(2).unwrap().position[0] + player.position[0];
            let dpy = player.shape.get(1).unwrap().position[1] + player.position[1];
            let upy = player.shape.get(0).unwrap().position[1] + player.position[1];

            if rpx > lbx {
                if dpy > uby && upy < dby {
                    reflect(self);
                }
            }
        }

        // check collision with opponent
        {
            let lox = opponent.shape.get(0).unwrap().position[0] + opponent.position[0];
            let doy = opponent.shape.get(1).unwrap().position[1] + opponent.position[1];
            let uoy = opponent.shape.get(0).unwrap().position[1] + opponent.position[1];
            let rox = opponent.shape.get(2).unwrap().position[0] + opponent.position[0];

            if lox <= rbx && rox >= lbx {
                if doy > uby && uoy < dby {
                    reflect(self);
                }
            }
        }

        {
            // check collision with ceilings
            if uby <= -1.0 || dby >= 1.0 {
                self.direction[1] = -self.direction[1];
            }

            if lbx <= -1.0 || rbx >= 1.0 {
                let new_ball = Ball::new();
                self.direction = new_ball.direction;
                self.position = new_ball.position;
                self.shape = new_ball.shape;
            }
        }
    }
}

impl Instanceable for Ball {
    fn get_instance_buffer(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<[InstanceInfo], PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>
    {
        let data = vec![InstanceInfo {
            offset: self.position,
        }];

        let e = CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, data).unwrap();
        e
    }
}

pub struct Opponent {
    pub position: [f32; 2],
    pub shape: Vec<Vertex>,
}

impl Opponent {
    pub fn new() -> Self {
        Self {
            position: [0.95, 0.0],
            shape: get_player(),
        }
    }

    pub fn update_position(&mut self, ball: &Ball) {
        let mut movement = 0.0;
        if self.position[1] > ball.position[1] {
            movement -= 0.01;
        } else if self.position[1] < ball.position[1] {
            movement += 0.01;
        }

        let upy = self.shape.get(0).unwrap().position[1] + self.position[1];
        let dpy = self.shape.get(1).unwrap().position[1] + self.position[1];

        if upy <= -1.0 && movement < 0.0 {
            movement = 0.0;
        } else if dpy >= 1.0 && movement > 0.0 {
            movement = 0.0;
        }

        self.position[1] += movement;
    }
}

impl Instanceable for Opponent {
    fn get_instance_buffer(
        &self,
        device: Arc<Device>,
    ) -> Arc<CpuAccessibleBuffer<[InstanceInfo], PotentialDedicatedAllocation<StdMemoryPoolAlloc>>>
    {
        let data = vec![InstanceInfo {
            offset: self.position,
        }];

        let e = CpuAccessibleBuffer::from_iter(device, BufferUsage::all(), false, data).unwrap();
        e
    }
}
