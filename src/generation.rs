//! Defines a structure for fast triangle mesh generation

use std::convert::Into;
use std::iter::FromIterator;
use crate::mesh::Mesh;
use crate::mesh::math::*;


type Vec2 = Vector2<f64>;

/// Structure intended to hold the points and faces during the generation
///
/// The generation functions are all methods of this class, and can be called independently of what is already in the shape, the common structure is only there to economize the memory reallocations.
/// The methods that are generating points and faces only from arguments (and so independently of what is already in the Shape) are using the name of the operation they represents (eg: `triangulation`), whereas the methods that use existing points or faces to operate on are called using verbs (eg: `triangulate`)
#[derive(Debug, Default)]
pub struct Shape {
	/// buffer of points
	pub points: Vec<Vec3>,		
	/// triangle indices in the buffer of points
	pub faces: Vec<[u32; 3]>,	
}


impl Into<Mesh> for Shape {
	fn into(self) -> Mesh {
		// TODO ideally we would do this:
		// Mesh::new(self.faces, self.points)
		// but the current Mesh new function is asking an owned storage, with splited vector and face components
		let mut faces = Vec::with_capacity(self.faces.len()*3);
		for face in self.faces {
			faces.extend_from_slice(&face);
		}
		let mut points = Vec::with_capacity(self.points.len()*3);
		for point in self.points {
			points.extend_from_slice((point.as_ref() as &[f64; 3]).as_ref());
		}
		Mesh::new(faces, points)
	}
}

impl Shape {
	/// create with empty buffers
	pub fn new() -> Self {
		Shape::default()
	}
	
	/// insert a surface defined by its outline
	/// The ouline is considered closed (ie. the last point is connected to the first)
	pub fn triangulation(&mut self, outline: &[Vec3]) -> &mut Self {
		self.points.extend_from_slice(outline);
		self.triangulate((0 .. outline.len() as u32).collect());
		self
	}
	
	/// create the triangles for a surface defined by its `outline`. 
	/// The ouline is considered closed (ie. the last point is connected to the first)
	pub fn triangulate(&mut self, closed_outline: Vec<u32>) -> &mut Self {
		let mut outline = closed_outline;	// move it to mutable
		
		// fast-way for the trivial case
		if outline.len() == 3 {
			self.faces.push([outline[0], outline[1], outline[2]]);
		}
		
		// project all the points on the face approximate plane
		let (x,y,_) = base_from_points(&mut outline.iter().map(|i| self.points[*i as usize]));
		let mut pts = Vec::from_iter(	outline.iter()
											.map(|i| self.points[*i as usize])
											.map(|p| Vec2::new(p.dot(x), p.dot(y)))
											);
		
		'triangulation: while outline.len() > 2 {
		
			// find the thinest triangle along an edge from outline that doesn't contains other points
			let mut tinyest_index = 0;
			let mut tinyest_surf = 0.;
			'search: for i in 0 .. outline.len() {
				// adjacent points to i
				let i2 = if i==outline.len()-1 {0} else {i+1};
				let i1 = if i==0 {outline.len()-1} else {i-1};
				// adjacents segments to point i
				let c = pts[i];
				let a = pts[i2] - c;
				let b = pts[i1] - c;
				
				let surf = a.perp_dot(b);
				
				// if surface is negative, then pass (the surface is then outside the outline)
				if surf <= tinyest_surf { continue 'search; }
				
				let mat = Matrix2::from_cols(a, b).invert().unwrap();
				
				// check that there is not point of the ouline inside the triangle
				'interiors: for (j,&p) in pts.iter().enumerate() {
					if j==i1 || j==i || j==i2	{ continue 'interiors; }
					let params = mat * (p - c);
					let (u,v) = (params[0], params[1]);
					if	0. < u && u < 1. && 0. < v && v <= 1.-u && v != 1.
						{ continue 'search; }
				}
				tinyest_index = i;
				tinyest_surf = surf;
			}
			assert!(tinyest_surf > 0., format!("no more feasible triangles in {:?}", pts));
			
			// create the triangle and update the outline
			let i = tinyest_index;
			// adjacent points to i
			let i2 = if i==outline.len()-1 {0} else {i+1};
			let i1 = if i==0 {outline.len()-1} else {i-1};
			self.faces.push([outline[i2] as u32, outline[i] as u32, outline[i1] as u32]);
			outline.remove(i);
			pts.remove(i);
		}
		self
	}
	
	/// extrude a line into a lateral surface
	pub fn extrusion(&mut self, line: &[Vec3], displt: Vec3) -> &mut Self {
		self.points.reserve(line.len()*2);
		self.faces.reserve((line.len()-1)*2);
		
		let num = line.len() as u32;
		let istart = self.points.len() as u32;
		// get the points in
		for point in line {
			self.points.push(*point);
			self.points.push(point+displt);
		}
		// add the triangles
		for i in istart+1 .. istart+num {
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
		let mut istart = self.points.len() as u32;
		self.points.extend_from_slice(line);
		
		for segt in 0 .. segments {
			let amount = segt as f64 / segments as f64;
			for pt in line.iter() {
				self.points.push(transform(amount, *pt));
			}
			for i in istart+1 .. istart+num {
				self.faces.push([i-1, i, i+num]);
				self.faces.push([i-1, i+num, i+num-1]);
			}
			istart += line.len() as u32;
		}
		self
 	}
 	
 	/// create a revolution surface from the line, around the axis
 	///
 	pub fn revolution(&mut self, line: &[Vec3], segments: usize, axis: Vec3, angle: f64) -> &mut Self {
		self.extrans(line, segments, &|amount, pt| Quaternion::from_axis_angle(axis, Rad(angle*amount)).rotate_vector(pt));
		self
 	}
 	
 	pub fn is_valid(&self) -> bool {
		// TODO use an error as return value
		let maxindex = self.points.len() as u32;
		for face in self.faces.iter() {
			for &i in face.iter() {
				if i > maxindex { return false; }
			}
		}
		true
 	}
}


fn base_from_points(points: &mut dyn Iterator<Item=Vec3>) -> (Vec3, Vec3, Vec3) {
	let err = "not enough vectors to cet a base, needs 3 differents";
	let o = points.next().expect(err);
	let x = (points.next().expect(err) - o).normalize();
	let mut z = Vec3::zero();
	while z.magnitude() < 1e-4 {
		z = x.cross(points.next().expect(err) - o).normalize();
	}
	let y = z.cross(x);
	(x,y,z)
}


#[cfg(test)]
mod tests {
	use super::*;
	
	#[test]
	fn test_triangulation() 
	{
		let mut shape = Shape::new();
		shape.triangulation(&vec![
			Vec3::new(0., 0., 0.), 
			Vec3::new(1., 0., 0.), 
			Vec3::new(1., 1., 0.), 
			Vec3::new(2., 1., 0.),
			Vec3::new(0., 2., 0.),
			]);
		assert_eq!(shape.faces.len(), 3);
		assert!(shape.is_valid());
	}
	
	#[test]
	fn test_extrusion() {
		let mut shape = Shape::new();
		shape.extrusion(&vec![
			Vec3::new(0., 0., 0.), 
			Vec3::new(1., 0., 0.), 
			Vec3::new(1., 1., 0.), 
			Vec3::new(0., 1., 0.),
			], Vec3::new(0., 0.5, 1.));
		
		assert_eq!(shape.points.len(), 8);
		assert_eq!(shape.faces.len(), 6);	// there is no face for the non-existing edge (0, 3)
		assert!(shape.is_valid());
	}
	
}
