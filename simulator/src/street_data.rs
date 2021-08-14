use serde::Deserialize;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;
use super::crossing::Crossing;



/// A struct representing the street network
///
/// The `StreetData` Struct itself holds a strong reference (`Rc` as opposed to `Weak`)
/// to the Crossings, while the Connections only hold weak references
/// to prevent reference cycles.
/// If the Connections held strong references, the memory wouldn't be cleaned
/// up when the StreetData goes out of scope, as the connections would form a
/// cycle
pub struct StreetData {
    crossings: Vec<Rc<RefCell<Crossing>>>,
}

#[derive(Debug, Clone)]
pub struct JsonError (String);
impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for JsonError {}

impl StreetData {
    /// creates a `StreetData` object from a `&str` formatted in a json-like way
    ///
    /// to see how the json must be formatted, look at the fields of
    /// `JsonCrossing` and `JsonRepresentation`
    fn from_json(json: &str) -> Result<StreetData, Box<dyn Error>> {
        // Generate object holding all the data, still formatted in json way
        let json_representation: JsonRepresentation = serde_json::from_str(json)?;
        let mut crossings: Vec<Rc<RefCell<Crossing>>> = Vec::new();    
        // generate all crossings
        for json_crossing in json_representation.crossings.iter() {
            let new_crossing = Crossing::new(json_crossing.is_io_node);
            crossings.push(Rc::new(RefCell::new(new_crossing)));
        }
        // connect the crossings
        for (i, json_crossing) in json_representation.crossings.iter().enumerate() {
            let c1 = crossings.get(i).unwrap();
            // form all the connections defined in `JsonCrossing.connected`
            for (connection_index, lanes) in json_crossing.connected.iter() {
                let c2 = crossings.get(*connection_index)
                    .ok_or("Invalid connection index in json")?;
                // Make sure the connection doesn't already exists
                if c1.borrow().get_connection(c2).is_some() {
                    return Err(
                        Box::new(
                            JsonError("Attempt to create the same connection multiple times".to_string())
                        )
                    )
                };
                // form the connection
                c1.borrow_mut().connect(c2, *lanes);
            }
        }
        Ok(
            StreetData {
                crossings,
            }
        )
    }
}

/// This is just used to deserialize the JSON File to
/// an object that can be conveniently used in 
/// `StreetData::from_json`
/// 
#[derive(Debug, Deserialize)]
struct JsonCrossing {
    traffic_lights: bool,
    is_io_node: bool,
    connected: Vec<(usize, u8)>,
}
#[derive(Debug, Deserialize)]
/// Just for Deserialisation
struct JsonRepresentation {
    crossings: Vec<JsonCrossing>
}

mod tests {
    use super::*;
    #[test]
    fn street_data_from_json() {
        let json: &str = r#"{"crossings": [{"traffic_lights": false, "is_io_node": false, "connected": [[1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[0, 1], [2, 1], [3, 1], [4, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [3, 1], [4, 1], [5, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[2, 1], [1, 1]]}, {"traffic_lights": false, "is_io_node": false, "connected": [[1, 1], [2, 1]]}, {"traffic_lights": false, "is_io_node": true, "connected": [[2, 1]]}]}"#;
        StreetData::from_json(json);
    }
}