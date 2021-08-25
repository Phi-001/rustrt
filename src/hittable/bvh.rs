use super::*;
use std::rc::Rc;

#[derive(Default)]
pub struct HittableList {
    objects: Vec<Hittable>,
    nodes: Vec<LinearBVHNode>,
}

impl HittableList {
    pub fn add(&mut self, object: Hittable) {
        self.objects.push(object);
    }

    pub fn hit(
        &self,
        ray: &Ray,
        t_min: Float,
        t_max: Float,
        interaction: &mut Interaction,
    ) -> bool {
        let inv_dir = Vector3::new(
            1.0 / ray.direction.x,
            1.0 / ray.direction.y,
            1.0 / ray.direction.z,
        );
        let dir_is_neg = [
            ray.direction.x < 0.0,
            ray.direction.y < 0.0,
            ray.direction.z < 0.0,
        ];
        let mut temp_interaction = Interaction::default();
        let mut hit_anything = false;
        let mut closest_so_far = t_max;

        let mut to_visit = 0;
        let mut current_node_index = 0;

        let mut nodes_to_visit = [usize::MAX; 64];

        loop {
            let node = self.nodes[current_node_index];
            if node
                .bounds
                .intersect(ray, &inv_dir, &dir_is_neg, closest_so_far, t_min)
            {
                if node.num_hittable > 0 {
                    for i in 0..node.num_hittable {
                        if let HittableOffset(offset) = node.offset {
                            if self.objects[offset + i].hit(
                                ray,
                                t_min,
                                closest_so_far,
                                &mut temp_interaction,
                            ) {
                                hit_anything = true;
                                closest_so_far = temp_interaction.t;
                            }
                        }
                    }
                    if to_visit == 0 {
                        break;
                    }
                    to_visit -= 1;
                    current_node_index = nodes_to_visit[to_visit];
                } else {
                    if dir_is_neg[node.axis] {
                        nodes_to_visit[to_visit] = current_node_index + 1;
                        if let SecondChildOffset(offset) = node.offset {
                            current_node_index = offset;
                        }
                    } else {
                        current_node_index += 1;
                        if let SecondChildOffset(offset) = node.offset {
                            nodes_to_visit[to_visit] = offset;
                        }
                    }
                    to_visit += 1;
                }
            } else {
                if to_visit == 0 {
                    break;
                }
                to_visit -= 1;
                current_node_index = nodes_to_visit[to_visit];
            }
        }

        *interaction = temp_interaction;

        hit_anything
    }

    pub fn init(&mut self) {
        let mut hittable_info = Vec::with_capacity(self.objects.len());
        for (i, hittable) in self.objects.iter().enumerate() {
            let bound = hittable.bound();
            hittable_info.push(BVHHittableInfo {
                hittable_number: i,
                centroid: bound.center(),
                bounds: bound,
            })
        }

        let mut total_nodes = 0;
        let mut ordered_hittables = vec![];
        let len = hittable_info.len();
        let root = self.recursive_build(
            &mut hittable_info,
            0,
            len,
            &mut total_nodes,
            &mut ordered_hittables,
        );

        self.objects = ordered_hittables;

        let mut offset = 0;
        self.nodes = vec![LinearBVHNode::default(); total_nodes];
        self.flatten_bvh(Rc::new(root), &mut offset);
    }

    fn flatten_bvh(&mut self, node: Rc<BVHBuildNode>, offset: &mut usize) -> usize {
        let mut linear_node = LinearBVHNode {
            bounds: node.bounds,
            ..Default::default()
        };
        let my_offset = *offset;
        *offset += 1;
        if node.num_hittable > 0 {
            linear_node.offset = HittableOffset(node.first_hittable_offset);
            linear_node.num_hittable = node.num_hittable;
        } else {
            linear_node.axis = node.split_axis;
            linear_node.num_hittable = 0;
            self.flatten_bvh(node.children.as_ref().unwrap()[0].clone(), offset);
            linear_node.offset = SecondChildOffset(
                self.flatten_bvh(node.children.as_ref().unwrap()[1].clone(), offset),
            );
        }
        self.nodes[my_offset] = linear_node;
        my_offset
    }

    #[allow(clippy::needless_range_loop)]
    fn recursive_build(
        &self,
        hittable_info: &mut Vec<BVHHittableInfo>,
        start: usize,
        end: usize,
        total_nodes: &mut usize,
        ordered_hittables: &mut Vec<Hittable>,
    ) -> BVHBuildNode {
        *total_nodes += 1;

        let mut bounds = Bounds3::default();

        for i in start..end {
            bounds = Bounds3::union(&bounds, &hittable_info[i].bounds);
        }

        let num_hittables = end - start;

        if num_hittables == 1 {
            let first_hittable = ordered_hittables.len();
            let hittable_num = hittable_info[start].hittable_number;
            ordered_hittables.push(self.objects[hittable_num].clone());
            BVHBuildNode::init_leaf(first_hittable, num_hittables, &bounds)
        } else {
            let mut centroid_bounds = Bounds3::default();
            for i in start..end {
                centroid_bounds =
                    Bounds3::union_point(&centroid_bounds, &hittable_info[i].centroid);
            }

            let dimension = centroid_bounds.maximum_extent();

            let mut middle = (start + end) / 2;

            if (centroid_bounds.p_min[dimension] - centroid_bounds.p_max[dimension]).abs()
                < Float::EPSILON
            {
                let first_hittable = ordered_hittables.len();
                for i in start..end {
                    let hittable_num = hittable_info[i].hittable_number;
                    ordered_hittables.push(self.objects[hittable_num].clone());
                }
                BVHBuildNode::init_leaf(first_hittable, num_hittables, &bounds)
            } else {
                if num_hittables < 4 {
                    hittable_info[start..end].select_nth_unstable_by(middle - start, |a, b| {
                        Float::partial_cmp(&a.centroid[dimension], &b.centroid[dimension])
                            // arbitrary
                            // to stop NaNs from causing problems but it hopefully shouldn't happen
                            .unwrap_or(std::cmp::Ordering::Less)
                    });
                } else {
                    // use SAH with buckets
                    const NUM_BUCKETS: usize = 12;
                    let mut buckets = [BucketInfo::default(); NUM_BUCKETS];

                    for i in start..end {
                        let mut b = ((NUM_BUCKETS as Float)
                            * centroid_bounds.offset(&hittable_info[i].centroid)[dimension])
                            as usize;

                        if b == NUM_BUCKETS {
                            b = NUM_BUCKETS - 1
                        }

                        buckets[b].count += 1;
                        buckets[b].bounds =
                            Bounds3::union(&buckets[b].bounds, &hittable_info[i].bounds);
                    }

                    const COST_HITTABLE: Float = 1.0;
                    const COST_AABB: Float = 0.9;

                    let mut cost = [0.0; NUM_BUCKETS - 1];

                    for i in 0..NUM_BUCKETS - 1 {
                        let mut b1 = Bounds3::default();
                        let mut b2 = Bounds3::default();
                        let mut count1 = 0;
                        let mut count2 = 0;
                        for j in 0..=i {
                            b1 = Bounds3::union(&b1, &buckets[j].bounds);
                            count1 += buckets[j].count;
                        }
                        for j in (i + 1)..NUM_BUCKETS {
                            b2 = Bounds3::union(&b2, &buckets[j].bounds);
                            count2 += buckets[j].count;
                        }

                        let count1 = count1 as Float;
                        let count2 = count2 as Float;

                        cost[i] = COST_AABB
                            + COST_HITTABLE
                                * (count1 * b1.surface_area() + count2 * b2.surface_area())
                                / bounds.surface_area();
                    }

                    let (min_cost_bucket, &min_cost) = cost
                        .iter()
                        .enumerate()
                        .min_by(|(_, a), (_, b)| {
                            Float::partial_cmp(a, b).unwrap_or(std::cmp::Ordering::Less)
                        })
                        .unwrap();

                    let leaf_cost = num_hittables as Float;

                    const MAX_HITTABLES_IN_NODE: usize = 255;

                    if num_hittables > MAX_HITTABLES_IN_NODE || min_cost < leaf_cost {
                        let (left, right): (Vec<_>, Vec<_>) =
                            hittable_info[start..end].iter().partition(|info| {
                                let mut b = ((NUM_BUCKETS as Float)
                                    * centroid_bounds.offset(&info.centroid)[dimension])
                                    as usize;

                                if b == NUM_BUCKETS {
                                    b = NUM_BUCKETS - 1
                                }
                                b <= min_cost_bucket
                            });
                        middle = start + left.len();
                        hittable_info[start..middle].copy_from_slice(&left);
                        hittable_info[middle..end].copy_from_slice(&right);
                    } else {
                        let first_hittable = ordered_hittables.len();
                        for i in start..end {
                            let hittable_num = hittable_info[i].hittable_number;
                            ordered_hittables.push(self.objects[hittable_num].clone());
                        }
                        return BVHBuildNode::init_leaf(first_hittable, num_hittables, &bounds);
                    }
                }
                BVHBuildNode::init(
                    dimension,
                    Rc::new(self.recursive_build(
                        hittable_info,
                        start,
                        middle,
                        total_nodes,
                        ordered_hittables,
                    )),
                    Rc::new(self.recursive_build(
                        hittable_info,
                        middle,
                        end,
                        total_nodes,
                        ordered_hittables,
                    )),
                )
            }
        }
    }
}

#[derive(Default, Clone, Copy)]
struct BucketInfo {
    count: usize,
    bounds: Bounds3,
}

#[derive(Clone, Copy)]
struct BVHHittableInfo {
    hittable_number: usize,
    centroid: Point3,
    bounds: Bounds3,
}

#[derive(Default, Debug)]
struct BVHBuildNode {
    bounds: Bounds3,
    children: Option<[Rc<BVHBuildNode>; 2]>,
    split_axis: usize,
    first_hittable_offset: usize,
    num_hittable: usize,
}

impl BVHBuildNode {
    fn init_leaf(first: usize, num: usize, bounds: &Bounds3) -> BVHBuildNode {
        BVHBuildNode {
            first_hittable_offset: first,
            num_hittable: num,
            bounds: *bounds,
            children: None,
            split_axis: 0,
        }
    }

    fn init(axis: usize, child1: Rc<BVHBuildNode>, child2: Rc<BVHBuildNode>) -> BVHBuildNode {
        BVHBuildNode {
            first_hittable_offset: 0,
            num_hittable: 0,
            bounds: Bounds3::union(&child1.bounds, &child2.bounds),
            children: Some([child1, child2]),
            split_axis: axis,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum BVHOffset {
    HittableOffset(usize),
    SecondChildOffset(usize),
}

use BVHOffset::*;

impl Default for BVHOffset {
    fn default() -> BVHOffset {
        HittableOffset(0)
    }
}

#[derive(Clone, Copy, Default, Debug)]
struct LinearBVHNode {
    bounds: Bounds3,
    offset: BVHOffset,
    num_hittable: usize, // If 0, interior node
    axis: usize,
}
