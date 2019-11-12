use std::convert::Into;
use crate::mesh::Mesh;
use crate::mesh::math::*;


/// structure intended to hold the points and faces during the generation
/// the generation functions are all methods of this class, and can be called independently of what is already in the shape, the common structure is only there to economize the memory reallocations
#[derive(Debug, Default)]
pub struct Shape {
	pub points:	Vec<Vec3>,
	pub faces: Vec<[u32; 3]>,
}


// TODO
// impl Into<Mesh> for Shape {
// 	fn into(self) -> Mesh {
// 		Mesh::new(self.faces, self.points)
// 	}
// }

impl Shape {
	fn new() -> Self {
		Shape::default()
	}
	
	/// extrude a line into a lateral surface
	pub fn extrusion(&mut self, line: &[Vec3], displt: Vec3) -> &mut Self {
		self.points.reserve(line.len()*2);
		self.faces.reserve((line.len()-1)*2);
		
		let num = line.len() as u32;
		let istart = self.points.len();
		// get the points in
		for point in line {
			self.points.push(*point);
			self.points.push(point+displt);
		}
		// add the triangles
		for i in 1 .. num {
			self.faces.push([i-1, i,   i+1]);
			self.faces.push([i-1, i+1, i+2]);
		}
		self
	}
	
	/// extrude a line into a lateral surface, proceeding by steps.
	/// each point the result of `transform(step/segments, line_point)`
	///
 	pub fn extrans(&mut self, line: &[Vec3], segments: usize, transform: &dyn Fn(f64,Vec3) -> Vec3) -> &mut Self {
		self.points.reserve(line.len() * (segments+1));
		self.faces.reserve((line.len()-1) * segments * 2);
		
		let num = line.len() as u32;
		let mut istart = self.points.len();
		self.points.extend_from_slice(line);
		
		for segt in 0 .. segments {
			let amount = segt as f64 / segments as f64;
			for pt in line.iter() {
				self.points.push(transform(amount, *pt));
			}
			for i in 1 .. num {
				self.faces.push([i-1, i, i+num]);
				self.faces.push([i-1, i+num, i+num-1]);
			}
			istart += line.len();
		}
		self
 	}
 	
 	/// create a revolution surface for the line, around the axis
 	///
 	pub fn revolution(&mut self, line: &[Vec3], segments: usize, axis: Vec3, angle: f64) -> &mut Self {
		self.extrans(line, segments, &|amount, pt| Quaternion::from_axis_angle(axis, Rad(angle*amount)).rotate_vector(pt));
		self
 	}
}
