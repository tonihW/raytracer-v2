use crate::intersection::Intersection;
use crate::material::Material;
use crate::utils::EPSILON;
use crate::vertex::Vertex;
use bvh::aabb::{AABB, Bounded};
use bvh::bounding_hierarchy::BHShape;
use bvh::ray::Ray;
use glam::{Vec3, Vec2};

pub struct Triangle {
    pub vrt: [Vertex; 3],
    pub mat: Material,
    pub node_idx: usize, // for BVH
}

impl Triangle {
    /**
     * Uses the Möller-Trumbore intersection algorithm
     * Reference: http://www.graphics.cornell.edu/pubs/1997/MT97.html
     */
    pub fn intersect(&self, ray: &Ray) -> Option<Intersection> {
        // calculate triangle edge vectors
        let edge_a = self.vrt[1].pos - self.vrt[0].pos;
        let edge_b = self.vrt[2].pos - self.vrt[0].pos;

        // solve the equation for t (distance)
        let p = ray.direction.cross(edge_b);
        let d = edge_a.dot(p);

        if d < EPSILON {
            return None;
        }

        let inv_d = 1.0 / d;
        let t = ray.origin - self.vrt[0].pos;
        let u = t.dot(p) * inv_d;

        if u < 0.0 || u > 1.0 {
            return None;
        }

        let q = t.cross(edge_a);
        let v = ray.direction.dot(q) * inv_d;

        if v < 0.0 || u + v > 1.0 {
            return None;
        }

        let t = edge_b.dot(q) * inv_d;

        if t < EPSILON {
            return None;
        }

        // calculate hit position
        let pos = ray.origin + ray.direction * t;

        // calculate barycentric coords for normals and texture coords
        let (b0, b1, b2) = self.barycentric(&p, &edge_a, &edge_b);

        return Some(Intersection {
            t,
            pos,
            nrm: self.vrt[0].nrm + b1 * (self.vrt[1].nrm - self.vrt[0].nrm) + b2 * (self.vrt[2].nrm - self.vrt[0].nrm),
            tex: self.vrt[0].tex * b0 + self.vrt[1].tex * b1 + self.vrt[2].tex * b2,
            mat: self.mat,
        });
    }

    /**
     * Reference: https://www.pbr-book.org
     */
    pub fn barycentric(&self, p: &Vec3, edge_a: &Vec3, edge_b: &Vec3) -> (f32, f32, f32) {
        // calculate line from target point to v0
        let w = *p - self.vrt[0].pos;

        // calculate perpendicular vectors between edges and calculated line
        let v_cross_w = edge_b.cross(w);
        let u_cross_w = edge_a.cross(w);
        let u_cross_v = edge_a.cross(*edge_b);

        // calculate barycentric coordinates for target point inside the triangle
        let denom = u_cross_v.length();
        let r = v_cross_w.length() / denom;
        let t = u_cross_w.length() / denom;

        return (1.0 - r - t, r, t);
    }
}

impl Bounded for Triangle {
    fn aabb(&self) -> AABB {
        let min = self.vrt[0].pos.min(self.vrt[1].pos.min(self.vrt[2].pos));
        let max = self.vrt[0].pos.max(self.vrt[1].pos.max(self.vrt[2].pos));
        return AABB::with_bounds(min, max);
    }
}

impl BHShape for Triangle {
    fn set_bh_node_index(&mut self, index: usize) {
        self.node_idx = index;
    }

    fn bh_node_index(&self) -> usize {
        self.node_idx
    }
}
