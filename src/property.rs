//Property system
//
use std::fmt;

type ElementId = usize;

#[derive(Debug)]
struct Property {
    name: String,
    parent_id: ElementId,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: String,
    property_id: ElementId,
}

enum Watch {
    DirWatch(u32),
    PropertyWatch(u32),
    AttributeWatch(u32),
}

struct Watcher {
    watch: Watch,
}

pub struct PropertyServer {
    properties: Vec<Property>,
    attributes: Vec<Attribute>,
    watchers: Vec<Watcher>,
}

#[derive(Debug)]
pub enum PropertyError {
    NotFound,
}

impl fmt::Display for PropertyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            NotFound => write!(f, "Property not found"),
        }
    }
}

impl std::error::Error for PropertyError {
    fn description(&self) -> &str {
        match self {
            NotFound => "Property not found",
        }
    }
}

impl PropertyServer {
    pub fn new() -> PropertyServer {
        let mut server = PropertyServer {
            properties: Vec::new(),
            attributes: Vec::new(),
            watchers: Vec::new(),
        };
        //The first property is the dummy root property just so that we get an index 0 for the root.
        server.properties.push(Property {
            name: "DUMMY_ROOT_PROP".to_string(),
            parent_id: 0,
        });
        server
    }

    pub fn get(
        &self,
        property: &str,
        attrib: Option<&str>,
    ) -> Result<Vec<(String, String)>, PropertyError> {
        let vec: Vec<&str> = property.split('.').collect();
        println!("Vector is {:?}", vec);
        let mut err = false;
        let mut property_id = 0;
        for prop_path in vec.iter() {
            match self.check_path(prop_path, property_id) {
                Ok(id) => {
                    property_id = id;
                }
                Err(e) => {
                    err = true;
                    break;
                }
            }
        }

        if err {
            Err(PropertyError::NotFound)
        } else {
            let res = self
                .attributes
                .iter()
                .filter_map(|x| {
                    if x.property_id == property_id
                        && if let Some(a) = attrib {
                            a == x.name
                        } else {
                            true
                        }
                    {
                        Some((x.name.clone(), x.value.clone()))
                    } else {
                        None
                    }
                })
                .collect();

            Ok(res)
        }
    }

    pub fn check_path(&self, name: &str, parent: ElementId) -> Result<ElementId, PropertyError> {
        if let Some(id) = self.properties.iter().position(|x| {
            //            println!("find : {:?}", &x.name);
            x.parent_id == parent && x.name == name
        }) {
            Ok(id)
        } else {
            Err(PropertyError::NotFound)
        }
    }

    pub fn set(
        &mut self,
        property: &str,
        attrib: Option<(&str, &str)>,
    ) -> Result<(), PropertyError> {
        let vec: Vec<&str> = property.split('.').collect();
        println!("Vector is {:?}", vec);
        let mut parent_id = 0;
        for prop_path in vec.iter() {
            parent_id = self.add_path(prop_path, parent_id);
        }

        if let Some((name, value)) = attrib {
            self.add_attrib(name, value, parent_id);
        }
        println!("Properties : {:?}", self.properties);
        println!("Attributes : {:?}", self.attributes);
        Ok(())
    }

    /// If the property already exists return the ElementID of the property, else
    /// create it and return its ElementID
    fn add_path(&mut self, name: &str, parent: ElementId) -> ElementId {
        if let Some(id) = self.properties.iter().position(|x| {
            //            println!("find : {:?}", &x.name);
            x.parent_id == parent && x.name == name
        }) {
            id
        } else {
            self.properties.push(Property {
                name: name.to_string(),
                parent_id: parent,
            });
            self.properties.len() - 1 as ElementId
        }
    }

    fn add_attrib(&mut self, name: &str, value: &str, property_id: ElementId) -> ElementId {
        if let Some(id) = self.attributes.iter().position(|x| {
            //            println!("find : {:?}", &x.name);
            x.property_id == property_id && x.name == name
        }) {
            self.attributes[id].value = value.to_string();
            id
        } else {
            self.attributes.push(Attribute {
                name: name.to_string(),
                value: value.to_string(),
                property_id: property_id,
            });
            self.attributes.len() - 1 as ElementId
        }
    }
}

#[test]
fn test_property_server() {
    let mut server = PropertyServer::new();

    server.set("root.service.property", None);
    server.set("root.service.property", Some(("attrib", "attrib_value")));
    server.set("root.service.property", Some(("attrib", "attrib_value1")));
    server.set("root.service.property", Some(("attrib1", "attrib_value2")));

    println!("Get {:?}", server.get("root.service.property", None));
    println!("Get {:?}", server.get("root.service.property", Some("attrib1")));
    println!("Get {:?}", server.get("root.service.property", Some("attrib1sdsd")));


}
