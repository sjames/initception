/*
    Copyright 2020 Sojan James
    Licensed under the Apache License, Version 2.0 (the "License");
    you may not use this file except in compliance with the License.
    You may obtain a copy of the License at
        http://www.apache.org/licenses/LICENSE-2.0
    Unless required by applicable law or agreed to in writing, software
    distributed under the License is distributed on an "AS IS" BASIS,
    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
    See the License for the specific language governing permissions and
    limitations under the License.
*/

//Property system
//
use std::fmt;

type ElementId = usize;

#[derive(Debug)]
struct Property {
    name: String,
    parent_id: ElementId,
    is_set: bool,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: String,
    property_id: ElementId,
}

enum WatchType {
    PropertyWatch(u32),
    AttributeWatch(u32),
}

enum WatchRule {
    OnCreate,
    OnChange,
    Equals(String, String),
}

struct Watcher<T> {
    watch: WatchType,
    rule: WatchRule,
    id: ElementId,
    context: T,
}

pub struct PropertyServer<T> {
    properties: Vec<Property>,
    attributes: Vec<Attribute>,
    watchers: Vec<Watcher<T>>,
    deleted_watchers: Vec<Watcher<T>>,
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

impl<T> PropertyServer<T> {
    pub fn new() -> PropertyServer<T> {
        let mut server = PropertyServer {
            properties: Vec::new(),
            attributes: Vec::new(),
            watchers: Vec::new(),
            deleted_watchers: Vec::new(),
        };
        //The first property is the dummy root property just so that we get an index 0 for the root.
        server.properties.push(Property {
            name: "DUMMY_ROOT_PROP".to_string(),
            parent_id: 0,
            is_set: false,
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

    fn check_path(&self, name: &str, parent: ElementId) -> Result<ElementId, PropertyError> {
        if let Some(id) = self.properties.iter().position(|x| {
            //            println!("find : {:?}", &x.name);
            x.parent_id == parent && x.name == name
        }) {
            Ok(id)
        } else {
            Err(PropertyError::NotFound)
        }
    }

    // create a watch. Take ownership of watch that will be returned when the watch has been triggered
    pub fn watch(
        &mut self,
        path: &str,
        watch: T,
        rule: WatchRule,
        watch_type: WatchType,
    ) -> Result<(), PropertyError> {
        let mut parent_id: ElementId = 0;

        for p in path.split('.') {
            parent_id = self.add_path(p, parent_id, false);
        }

        match rule {
            WatchRule::Equals(k, v) => {}
            WatchRule::OnChange => {}
            WatchRule::OnCreate => {}
        }

        Err(PropertyError::NotFound)
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
            parent_id = self.add_path(prop_path, parent_id, true);
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
    fn add_path(&mut self, name: &str, parent: ElementId, is_set: bool) -> ElementId {
        if let Some(id) = self
            .properties
            .iter()
            .position(|x| x.parent_id == parent && x.name == name)
        {
            // In case the property was already present because of a watch
            if is_set {
                self.properties[id].is_set = true;
            }
            id
        } else {
            self.properties.push(Property {
                name: name.to_string(),
                parent_id: parent,
                is_set: is_set,
            });
            self.properties.len() - 1 as ElementId
        }
    }

    fn add_attrib(&mut self, name: &str, value: &str, property_id: ElementId) -> ElementId {
        if let Some(id) = self
            .attributes
            .iter()
            .position(|x| x.property_id == property_id && x.name == name)
        {
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

    server.set("root.a.property", None);
    server.set("root.b.property", Some(("attrib1", "attrib_value1")));
    server.set("root.c.property", Some(("attrib2", "attrib_value2")));
    server.set("root.d.property", Some(("attrib3", "attrib_value3")));
    server.set("root.e.property", Some(("attrib4", "attrib_value4")));
    server.set("root.f.property", Some(("attrib5", "attrib_value5")));
    server.set("root.g.property", Some(("attrib6", "attrib_value6")));
    server.set("root.h.property", Some(("attrib7", "attrib_value7")));
    server.set("a.b.c.d.e.f.g.h.i", Some(("attribZZZZ", "BOOHOO")));

    /*
        println!("Get {:?}", server.get("root.service.property", None));
        println!(
            "Get {:?}",
            server.get("root.service.property", Some("attrib1"))
        );
        println!(
            "Get {:?}",
            server.get("root.service.property", Some("attrib1sdsd"))
        );
        println!("Get {:?}", server.get("a.b.c.d.e.f.g.h.i", None));

        // set a watch on when root.a.property gets created
        server.watch("root.a.property", (), WatchRule::OnCreate, Some(WatchType::PropertyWatch));

        // set a watch on when root.a.property changes
        server.watch("root.a.property", (), WatchRule::OnChange, Some(WatchType::PropertyWatch));

        // set a watch when the attribute key of a property becomes value
        server.watch("root.a.property", (), WatchRule::Equals("key","value", None);
    */
}
