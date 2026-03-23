use std::collections::{HashMap, HashSet};

use crate::ir::{TypeDef, TypeDefKind, TypeRef};

/// Detect cycles among types connected by direct `TypeRef::Named` edges and wrap
/// the offending references in `TypeRef::Boxed` so the generated Rust structs
/// have finite size.
///
/// "Direct" means the `Named` is not already behind indirection (`Vec`, `Option`,
/// `Map`). Non-required struct fields are Option-wrapped during codegen, so they
/// are already indirected and excluded from the graph.
pub fn box_recursive_types(types: &mut Vec<TypeDef>) {
    let graph = build_direct_edges(types);

    for type_def in types.iter_mut() {
        let owner = &type_def.name;
        match &mut type_def.kind {
            TypeDefKind::Struct(s) => {
                for base in &mut s.bases {
                    box_if_cyclic(base, owner, &graph);
                }
                for field in &mut s.fields {
                    if field.required {
                        box_if_cyclic(&mut field.type_ref, owner, &graph);
                    }
                }
            }
            TypeDefKind::Enum(e) => {
                for variant in &mut e.variants {
                    box_if_cyclic(&mut variant.type_ref, owner, &graph);
                }
            }
            TypeDefKind::Alias(type_ref) => {
                box_if_cyclic(type_ref, owner, &graph);
            }
            TypeDefKind::StringEnum(_) => {}
        }
    }
}

/// If `type_ref` is a bare `Named(target)` and `target` can reach `owner`
/// through direct edges, wrap it in `Boxed`.
fn box_if_cyclic(type_ref: &mut TypeRef, owner: &str, graph: &HashMap<String, HashSet<String>>) {
    if let TypeRef::Named(target) = type_ref {
        if can_reach(graph, target, owner) {
            let inner = std::mem::replace(type_ref, TypeRef::Any);
            *type_ref = TypeRef::Boxed(Box::new(inner));
        }
    }
}

/// BFS from `start` following direct Named edges. Returns true if `goal` is reachable.
fn can_reach(graph: &HashMap<String, HashSet<String>>, start: &str, goal: &str) -> bool {
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        if current == goal {
            return true;
        }
        if !visited.insert(current.to_string()) {
            continue;
        }
        if let Some(neighbours) = graph.get(current) {
            for next in neighbours {
                queue.push_back(next);
            }
        }
    }
    false
}

/// Build a directed graph of "direct Named" edges from the type definitions.
fn build_direct_edges(types: &[TypeDef]) -> HashMap<String, HashSet<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

    for type_def in types {
        let owner = &type_def.name;
        let edges = graph.entry(owner.clone()).or_default();

        match &type_def.kind {
            TypeDefKind::Struct(s) => {
                for base in &s.bases {
                    if let TypeRef::Named(target) = base {
                        edges.insert(target.clone());
                    }
                }
                for field in &s.fields {
                    if field.required {
                        if let TypeRef::Named(target) = &field.type_ref {
                            edges.insert(target.clone());
                        }
                    }
                }
            }
            TypeDefKind::Enum(e) => {
                for variant in &e.variants {
                    if let TypeRef::Named(target) = &variant.type_ref {
                        edges.insert(target.clone());
                    }
                }
            }
            TypeDefKind::Alias(TypeRef::Named(target)) => {
                edges.insert(target.clone());
            }
            _ => {}
        }
    }

    graph
}
