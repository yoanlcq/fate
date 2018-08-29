pub type NodeID = u32;

#[derive(Debug, Default)]
struct LoadedGltf {
    xforms: HashMap<NodeID, Transform<f32, f32, f32>>,
    parents: HashMap<NodeID, NodeID>,
    meshes: HashMap<NodeID, GltfMesh>,
}

#[derive(Debug)]
struct GltfMesh {
}

impl LoadedGltf {
    fn parse_mesh(&mut self, gltf: &gltf::Document, node_id: NodeID, mesh: &gltf::Mesh) {
        for prim in mesh.primitives() {
            if let Some(indices) = prim.indices() {
                match (indices.dimensions(), indices.data_type()) {
                    (gltf::accessor::Dimensions::Scalar, gltf::accessor::DataType::U16) => {
                        assert_eq!(indices.size(), 2);
                    },
                    _ => unimplemented!(),
                }
                let mut offset = indices.offset(); // In bytes
                {
                    let view = indices.view(); // Buffer view
                    assert_eq!(view.stride(), None);
                    offset += view.offset();
                    // TODO: Find data from pre-loaded buffers. We probably only care about the index, not the source
                    /*
                    let data = match view.buffer().source() {
                        gltf::buffer::Source::Bin => (),
                        gltf::buffer::Source::Uri(uri) => (),
                    };
                    */
                }
                indices.count(); // Nb components
                // TODO: Index into data.
            }
            for (semantic, data) in prim.attributes() {
                match semantic {
                    gltf::Semantic::Positions => (),
                    gltf::Semantic::Normals => (),
                    gltf::Semantic::Tangents => unimplemented!(),
                  | gltf::Semantic::Colors(attr)
                  | gltf::Semantic::TexCoords(attr)
                  | gltf::Semantic::Joints(attr)
                  | gltf::Semantic::Weights(attr)
                        => unimplemented!(),
                }
            }
            match prim.mode() {
                gltf::mesh::Mode::Triangles => (),
              | gltf::mesh::Mode::Points
              | gltf::mesh::Mode::Lines
              | gltf::mesh::Mode::LineLoop
              | gltf::mesh::Mode::LineStrip
              | gltf::mesh::Mode::TriangleStrip
              | gltf::mesh::Mode::TriangleFan
                    => unimplemented!(),
            }
        }
    }
    fn node_transform(node: &gltf::Node) -> Transform<f32, f32, f32> {
        let (position, orientation, scale) = node.transform().decomposed();
        Transform {
            position: position.into(),
            orientation: Vec4::from(orientation).into(),
            scale: scale.into(),
        }
    }
    fn parse_node(&mut self, gltf: &gltf::Document, node: &gltf::Node, parent: Option<NodeID>) {
        let node_id = node.index() as NodeID;
        self.xforms.insert(node_id, Self::node_transform(node));
        if let Some(parent) = parent {
            self.parents.insert(node_id, parent);
        }
        if let Some(mesh) = node.mesh() {
            self.parse_mesh(gltf, node_id, &mesh);
        }
        for child in node.children() {
            self.parse_node(gltf, &child, Some(node_id));
        }
    }
    // Order:
    // - Hierarchy
    // - Vertex buffers + topology
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let (document, buffers, images) = gltf::import(path).unwrap();
        assert_eq!(buffers.len(), document.buffers().count());
        assert_eq!(images.len(), document.images().count());

        let mut slf = Self::default();
        for node in document.default_scene().unwrap().nodes() {
            slf.parse_node(&document, &node, None);
        }
        Ok(slf)
    }
}


