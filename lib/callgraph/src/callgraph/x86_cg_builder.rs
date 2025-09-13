use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use capstone::InsnGroupId;
use crate::callgraph::{get_compilation_unit_directories, get_procedures_for_compilation_unit, CallGraphBuilder, CompilationInfo, InvocationFinder};
use crate::{dwarf_utils, CallGraph, Context, Procedure};
use fallible_iterator::FallibleIterator;

/// Struct able to build a callgraph from an x86 binary
pub struct X86CallGraphBuilder<P, I, F> {
    pub(crate) invocation_finders: Vec<Box<dyn InvocationFinder<P, I, F>>>,
}


impl<PMetadata: Default, IMetadata: Default, FMetadata: Default> CallGraphBuilder<PMetadata, IMetadata, FMetadata>
for X86CallGraphBuilder<PMetadata, IMetadata, FMetadata>
{
    /// Function building the full call graph from the information in `ctx`.
    fn build_call_graph(&self, ctx: &Context) -> CallGraph<PMetadata, IMetadata, FMetadata> {
        // Initialize empty fields for callgraph
        let mut graph = petgraph::stable_graph::StableGraph::new();
        // Index mapping procedure start addresses to their index in the graph
        let mut proc_index = HashMap::new();
        // Index mapping call/jump instruction addresses to the index of their enclosing procedure in the graph
        let mut call_index = HashMap::new();

        // Fill fields for CallGraph
        let call_graph: CallGraph<PMetadata, IMetadata, FMetadata> = {
            let compilation_unit_dirs = get_compilation_unit_directories(ctx);
            let rust_version = dwarf_utils::get_rust_version(ctx);

            // Iterator over compilation units
            ctx.dwarf_info.units()
                // Map all compilation units to their respective procedures
                .map(|unit_header| {
                    Ok(get_procedures_for_compilation_unit(ctx, &compilation_unit_dirs, unit_header))
                })
                // Flatten Vec<Vec<Procedure>> to Vec<Procedure>
                .fold(vec!(), |mut vec: Vec<Procedure<PMetadata>>, mut elem| {
                    vec.append(&mut elem);
                    Ok(vec)
                })
                .expect("Failed to flatten")
                .into_iter()
                // Add all nodes to the graph, and all (addr, index) pairs to the proc_index map
                .for_each(|procedure| {
                    let address = procedure.start_address;
                    let idx = graph.add_node(Rc::new(RefCell::new(procedure)));

                    // https://github.com/aquynh/capstone/blob/0de0c8b49dba478759eccabb0c9caddc2b653375/include/x86.h#L1567
                    let group_calls = InsnGroupId(2);
                    let group_jumps = InsnGroupId(1);

                    // Add every call instruction of a procedure to the address to index map.
                    graph[idx].borrow().disassembly.iter()
                        .filter(|insn| {
                            ctx.capstone.insn_group_ids(insn).unwrap().any(|id| id == group_calls || id == group_jumps)
                        })
                        .for_each(|insn| {
                            call_index.insert(insn.address(), idx); });

                    proc_index.insert(address, idx);
                });

            self.invocation_finders.iter().for_each(|finder| {
                finder.find_invocations(
                    &mut graph,
                    &mut proc_index,
                    &mut call_index,
                    ctx,
                    CompilationInfo {
                        compilation_dirs: &compilation_unit_dirs,
                        rust_version: &rust_version.as_ref().cloned().unwrap_or_default(),
                    },
                )
            });

            CallGraph {
                graph,
                proc_index,
                call_index,
            }
        };

        call_graph
    }
}