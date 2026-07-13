use std::{collections::HashMap, hash::Hash};

#[derive(Debug, PartialEq, Eq)]
pub enum TopologicalSortError<N> {
    DependencyCycle { cycle: Vec<N> },
}

impl<N: std::fmt::Display> std::fmt::Display for TopologicalSortError<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TopologicalSortError::DependencyCycle { cycle } => {
                let cycle_str: Vec<String> = cycle.iter().map(|n| n.to_string()).collect();
                write!(f, "Dependency cycle detected: {}", cycle_str.join(" -> "))
            }
        }
    }
}

impl<N: std::fmt::Debug + std::fmt::Display> std::error::Error for TopologicalSortError<N> {}

pub fn topological_sort<N, F, I>(
    nodes: I,
    get_dependencies: F,
) -> Result<Vec<N>, TopologicalSortError<N>>
where
    N: Clone + Eq + Hash,
    I: IntoIterator<Item = N>,
    F: Fn(&N) -> Vec<N>,
{
    let mut order = Vec::new();
    let mut visited = HashMap::new();

    #[derive(Clone, Copy, PartialEq)]
    enum VisitState {
        Visiting,
        Visited,
    }

    fn dfs<N, F>(
        node: N,
        get_dependencies: &F,
        visited: &mut HashMap<N, VisitState>,
        order: &mut Vec<N>,
        path: &mut Vec<N>,
    ) -> Result<(), TopologicalSortError<N>>
    where
        N: Clone + Eq + Hash,
        F: Fn(&N) -> Vec<N>,
    {
        match visited.get(&node) {
            Some(VisitState::Visited) => return Ok(()),
            Some(VisitState::Visiting) => {
                let start_idx = path.iter().position(|x| x == &node).unwrap_or(0);
                let mut cycle = path[start_idx..].to_vec();
                cycle.push(node);
                return Err(TopologicalSortError::DependencyCycle { cycle });
            }
            None => {}
        }

        visited.insert(node.clone(), VisitState::Visiting);
        path.push(node.clone());

        for dep in get_dependencies(&node) {
            dfs(dep, get_dependencies, visited, order, path)?;
        }

        path.pop();
        visited.insert(node.clone(), VisitState::Visited);
        order.push(node);

        Ok(())
    }

    let mut path = Vec::new();
    for node in nodes {
        if !visited.contains_key(&node) {
            dfs(
                node.clone(),
                &get_dependencies,
                &mut visited,
                &mut order,
                &mut path,
            )?;
        }
    }

    Ok(order)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topological_sort_simple() {
        // A -> B -> C
        let mut deps = HashMap::new();
        deps.insert("A", vec![]);
        deps.insert("B", vec!["A"]);
        deps.insert("C", vec!["B"]);

        let nodes = vec!["C", "B", "A"];
        let result = topological_sort(nodes, |n| deps.get(n).cloned().unwrap_or_default()).unwrap();
        assert_eq!(result, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_topological_sort_cycle() {
        // A -> B -> C -> A
        let mut deps = HashMap::new();
        deps.insert("A", vec!["B"]);
        deps.insert("B", vec!["C"]);
        deps.insert("C", vec!["A"]);

        let nodes = vec!["A", "B", "C"];
        let result = topological_sort(nodes, |n| deps.get(n).cloned().unwrap_or_default());
        assert!(result.is_err());
        match result.unwrap_err() {
            TopologicalSortError::DependencyCycle { cycle } => {
                assert_eq!(cycle.first(), cycle.last());
                assert_eq!(cycle.len(), 4);
            }
        }
    }
}
