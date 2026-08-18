use std::cmp::Reverse;

use bevy::{platform::collections::HashMap, prelude::*};
use priority_queue::PriorityQueue;

use crate::utils::hex::AxialCoordinates;

/// Spawn this component to find the best path between to points.
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Pathfinder {
    start: AxialCoordinates,
    goal: AxialCoordinates,
    #[reflect(ignore)]
    frontier: PriorityQueue<AxialCoordinates, Reverse<usize>>,
    cost_so_far: HashMap<AxialCoordinates, usize>,
    came_from: HashMap<AxialCoordinates, AxialCoordinates>,
    steps: usize,
}

#[derive(Debug, PartialEq)]
pub enum PathfindingState {
    Calculating,
    Completed(Vec<AxialCoordinates>),
    Failed,
}

impl Pathfinder {
    pub fn new(start: AxialCoordinates, goal: AxialCoordinates) -> Self {
        let mut frontier = PriorityQueue::new();
        frontier.push(start, Reverse(0));
        Self {
            start,
            goal,
            frontier,
            cost_so_far: HashMap::from([(start, 0)]),
            came_from: HashMap::new(),
            steps: 0,
        }
    }

    pub(super) fn calculate_step(&mut self) -> PathfindingState {
        let Some((current_coordinates, _current_priority)) = self.frontier.pop() else {
            return PathfindingState::Failed;
        };

        if current_coordinates == self.goal {
            return PathfindingState::Completed(self.to_path());
        }

        let current_cost = self.cost_so_far[&current_coordinates];

        for neighbor in current_coordinates.neighbors() {
            let new_cost = current_cost + 1;
            if self
                .cost_so_far
                .get(&neighbor)
                .is_none_or(|&current_cost| new_cost < current_cost)
            {
                self.cost_so_far.insert(neighbor, new_cost);
                let priority = new_cost + neighbor.distance(&self.goal);
                self.frontier.push(neighbor, Reverse(priority));
                self.came_from.insert(neighbor, current_coordinates);
            }
        }

        self.steps += 1;
        PathfindingState::Calculating
    }

    fn to_path(&self) -> Vec<AxialCoordinates> {
        let mut points = Vec::new();
        let mut next = self.goal;
        points.push(next);
        while let Some(point) = self.came_from.get(&next) {
            points.push(*point);
            next = *point;
        }
        points.reverse();
        points
    }
}


#[cfg(test)]
mod test {
    use crate::{
        pathfinding::{Pathfinder, PathfindingState},
        utils::hex::AxialCoordinates,
    };

    #[test]
    fn test_pathfinding() {
        let mut pathfinding = Pathfinder::new(AxialCoordinates::ZERO, AxialCoordinates::RIGHT);
        assert_eq!(pathfinding.calculate_step(), PathfindingState::Calculating);
        assert_eq!(
            pathfinding.calculate_step(),
            PathfindingState::Completed(vec![AxialCoordinates::ZERO, AxialCoordinates::RIGHT])
        );
    }

    #[test]
    fn test_longer_pathfinding() {
        let mut pathfinding = Pathfinder::new(AxialCoordinates::ZERO, AxialCoordinates::new(2, 0));
        assert_eq!(pathfinding.calculate_step(), PathfindingState::Calculating);
        assert_eq!(pathfinding.calculate_step(), PathfindingState::Calculating);
        assert_eq!(
            pathfinding.calculate_step(),
            PathfindingState::Completed(vec![
                AxialCoordinates::ZERO,
                AxialCoordinates::new(1, 0),
                AxialCoordinates::new(2, 0,)
            ])
        );
    }
}
