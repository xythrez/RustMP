use hwloc2::{CpuBindError, CpuBindFlags, ObjectType, Topology, TopologyObject};
use lazy_static::lazy_static;
use std::cmp::max;
use std::sync::Arc;

lazy_static! {
    static ref INSTANCE: Arc<SystemObject> = Arc::new(SystemObject::new());
}

/// Represents a system object.
///
/// The system object contains both hardware topology information as well as environment variable
/// information used by RustMP. Interactions with the OS such as setting process affinity should be
/// done through the system object.
///
/// Only a single instance of SystemObject may exist at a time. Use get_instance() to get a
/// thread-safe reference to the current SystemObject instance.
pub struct SystemObject {
    /// ThreadID to hwthread (PU) mapping for process binding
    cpu_bind_map: Vec<usize>,
    /// Total number of hwthreads (PUs) on the machine
    pub available_hwthreads: usize,
    /// Maximum number of threads to spawn for the RustMP thread pool
    pub max_num_threads: usize,
}

impl SystemObject {
    /// Instantiates a new SystemObject.
    ///
    /// Extra environment variables and hardware information used should be added here.
    ///
    /// SystemObject::new() should only be called by INSTANCE.
    fn new() -> SystemObject {
        // Assume that the machine uses a symmetric topology
        let topo = Topology::new().unwrap();
        let package_set = topo.objects_with_type(&ObjectType::Package).unwrap();
        let core_set = children_with_type(package_set[0], &ObjectType::Core).unwrap();

        // PUs per Core
        let pupco = children_with_type(core_set[0], &ObjectType::PU)
            .unwrap()
            .len();
        // Packages per Machine
        let papma = package_set.len();
        // Cores per Package
        let coppa = core_set.len();
        // PUs per Package
        let puppa = coppa * pupco;
        // PUs per Machine
        let available_hwthreads= papma * puppa;

        // The cpu_bind_map can be instantiated using environment variables if desired.
        // Possible interesting improvement would be building a cpu_bind_map using a syntax like
        // OpenMP's OMP_PLACES. But that is outside of this project's scope due to difficulty of
        // writing a parser.
        let cpu_bind_map= (0..available_hwthreads).map(|x| {
            let package_id = x / puppa;
            let package_offset = x % puppa;
            let core_id = package_offset % coppa;
            let core_offset = package_offset / coppa;
            return puppa * package_id + core_id * pupco + core_offset;
        }).collect::<Vec<usize>>();

        let max_num_threads = max(
            option_env!("RMP_NUM_THREADS")
                .unwrap_or("")
                .parse::<usize>()
                .unwrap_or(available_hwthreads),
            1,
        );
        SystemObject {
            cpu_bind_map,
            available_hwthreads,
            max_num_threads,
        }
    }

    /// Gets a thread-safe reference to the current SystemObject instance.
    pub fn get_instance() -> Arc<SystemObject> {
        INSTANCE.clone()
    }

    /// Binds the current thread to the corresponding hwthread specified in cpu_bind_map.
    ///
    /// Default binding rules are by core, then by hwthread on the same core, then by socket.
    ///
    /// Returns an error if the process failed to bind
    pub fn set_affinity(&self, tid: usize) -> Result<(), CpuBindError> {
        let mut topo = Topology::new().unwrap();
        let pu_vec = topo.objects_with_type(&ObjectType::PU).unwrap();
        let cpuset = pu_vec[self.cpu_bind_map[tid % self.available_hwthreads]]
            .cpuset()
            .unwrap();
        topo.set_cpubind(cpuset, CpuBindFlags::CPUBIND_THREAD)
    }
}

/// Recursively finds child topology objects of a defined type.
///
/// Returns None if no children of the type is found.
fn children_with_type<'a>(
    topo_obj: &'a TopologyObject,
    object_type: &ObjectType,
) -> Option<Vec<&'a TopologyObject>> {
    let mut objects: Vec<&TopologyObject> = Vec::new();

    for child in topo_obj.children() {
        if child.object_type() == *object_type {
            objects.push(&child);
        } else {
            match children_with_type(child, object_type) {
                Some(mut child_vec) => objects.append(&mut child_vec),
                None => (),
            }
        }
    }

    if objects.len() > 0 {
        Some(objects)
    } else {
        None
    }
}
