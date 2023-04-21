use glam::Vec3;

pub struct DirLight {
    pub direction: Vec3,
    pub emission: Vec3,
}

pub struct PointLight {
    pub position: Vec3,
    pub emission: Vec3,
    pub c: f32,
    pub l: f32,
    pub q: f32,
}

pub trait Light {
    fn eval_we(&self, p: &Vec3) -> Vec3;
    fn eval_le(&self, we: &Vec3) -> Vec3;
}

impl Light for DirLight {
    fn eval_we(&self, _p: &Vec3) -> Vec3 {
        return self.direction.normalize();
    }

    fn eval_le(&self, _we: &Vec3) -> Vec3 {
        return self.emission;
    }
}

impl Light for PointLight {
    fn eval_we(&self, p: &Vec3) -> Vec3 {
        return -(self.position - *p);
    }

    fn eval_le(&self, we: &Vec3) -> Vec3 {
        let d = we.length();
        let a = 1.0 / (self.c + self.l * d + self.q * d * d);
        return self.emission * a;
    }
}
