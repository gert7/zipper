use lazy_static::lazy_static;
use nbt::Value::*;
use yaml_rust::{Yaml, YamlLoader};

const NBT_ITRUE: nbt::Value = Byte(1i8);
const NBT_IFALSE: nbt::Value = Byte(0i8);

type NbtMap = nbt::Map<std::string::String, nbt::Value>;

fn nbt_bool(b: bool) -> nbt::Value {
    if b {
        NBT_ITRUE
    } else {
        NBT_IFALSE
    }
}

fn load_yaml_from_file<'a>(filename: &str) -> Vec<Yaml> {
    let fstr = std::fs::read_to_string(filename).unwrap();
    let yaml = YamlLoader::load_from_str(&fstr).unwrap();
    let doc = &yaml[0];
    println!("{:?}", doc);
    yaml
}

fn insert(map: &mut NbtMap, k: &str, v: nbt::Value) {
    map.insert(k.to_owned(), v);
}

fn ynbt_bool(map: &mut NbtMap, doc: &Yaml, k: &str) {
    println!("{:?}", doc);
    println!("{} {:?}", k, doc[k]);
    insert(map, k, nbt_bool(doc[k].as_bool().unwrap()));
}

fn ynbt_float(map: &mut NbtMap, doc: &Yaml, k: &str) {
    println!("{:?}", doc);
    println!("{} {:?}", k, doc[k]);
    insert(map, k, Float(doc[k].as_f64().unwrap() as f32));
}

fn ynbt_int(map: &mut NbtMap, doc: &Yaml, k: &str) {
    println!("{:?}", doc);
    println!("{} {:?}", k, doc[k]);
    insert(map, k, Int(doc[k].as_i64().unwrap() as i32));
}

fn ynbt_string(map: &mut NbtMap, doc: &Yaml, k: &str) {
    println!("{:?}", doc);
    println!("{} {:?}", k, doc[k]);
    insert(map, k, String(doc[k].as_str().unwrap().to_owned()));
}

fn registry_entry_generic(name: &str, id: i32, element: nbt::Value) -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert("name".to_owned(), String(name.to_owned()));
    default.insert("id".to_owned(), Int(id));
    default.insert("element".to_owned(), element);
    nbt::Value::Compound(default)
}

fn make_dimension_type(doc: &Yaml) -> nbt::Value {
    let mut default = nbt::Map::new();
    let d = &mut default;
    ynbt_bool(d, &doc, "piglin_safe");
    ynbt_bool(d, &doc, "natural");
    ynbt_float(d, &doc, "ambient_light");
    ynbt_string(d, &doc, "infiniburn");
    ynbt_bool(d, &doc, "respawn_anchor_works");
    ynbt_bool(d, &doc, "bed_works");
    ynbt_string(d, &doc, "effects");
    ynbt_bool(d, &doc, "has_raids");
    ynbt_int(d, &doc, "min_y");
    ynbt_int(d, &doc, "height");
    ynbt_int(d, &doc, "logical_height");
    ynbt_float(d, &doc, "coordinate_scale");
    ynbt_bool(d, &doc, "ultrawarm");
    ynbt_bool(d, &doc, "has_ceiling");
    nbt::Value::Compound(default)
}

fn default_dimension_type() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert("piglin_safe".to_owned(), NBT_ITRUE);
    default.insert("natural".to_owned(), NBT_ITRUE);
    default.insert("ambient_light".to_owned(), Float(1.0f32));
    default.insert(
        "infiniburn".to_owned(),
        String("minecraft:infiniburn".to_owned()),
    );
    default.insert("respawn_anchor_works".to_owned(), NBT_IFALSE);
    default.insert("has_skylight".to_owned(), NBT_ITRUE);
    default.insert("bed_works".to_owned(), NBT_ITRUE);
    default.insert(
        "effects".to_owned(),
        String("minecraft:overworld".to_owned()),
    );
    default.insert("has_raids".to_owned(), NBT_ITRUE);
    default.insert("min_y".to_owned(), Int(-2032i32));
    default.insert("height".to_owned(), Int(2032i32));
    default.insert("logical_height".to_owned(), Int(256i32));
    default.insert("coordinate_scale".to_owned(), Float(20.0f32));
    default.insert("ultrawarm".to_owned(), NBT_IFALSE);
    default.insert("has_ceiling".to_owned(), NBT_IFALSE);
    nbt::Value::Compound(default)
}

fn default_dimension_entry() -> nbt::Value {
    registry_entry_generic("minecraft:overworld", 0, default_dimension_type())
}

fn default_dimension_registry() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert(
        "type".to_owned(),
        String("minecraft:dimension_type".to_owned()),
    );

    let dimensions = std::fs::read_to_string("src/world/dimensions.yaml").unwrap();
    let dimensions = YamlLoader::load_from_str(&dimensions).unwrap();
    let dimensions = &dimensions[0];

    let mut count = 0;

    let mut dimension_list = Vec::new();
    let mut dimension_map: NbtMap = nbt::Map::new();
    for (k, dim) in dimensions.as_hash().unwrap() {
        println!("{:?}", dim);
        let dim_nbt = make_dimension_type(&dim);
        let dim_regent = registry_entry_generic(k.as_str().unwrap(), count, dim_nbt);
        dimension_list.push(dim_regent.clone());
        dimension_map.insert(k.as_str().unwrap().to_owned(), dim_regent);
        count += 1;
    }
    // default.insert("value".to_owned(), List(dimension_list));
    default.insert("value".to_owned(), Compound(dimension_map));

    // let mut dimension_map = nbt::Map::new();
    // dimension_map.insert("minecraft:overworld".to_owned(), default_dimension_entry());
    // default.insert("value".to_owned(), Compound(dimension_map));
    nbt::Value::Compound(default)
}

fn default_biome_properties() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert("precipitation".to_owned(), String("rain".to_owned()));
    default.insert("depth".to_owned(), Float(0.0f32));
    default.insert("temperature".to_owned(), Float(1.0f32));
    default.insert("scale".to_owned(), Float(1.0f32));
    default.insert("downfall".to_owned(), Float(0.5f32));
    default.insert("category".to_owned(), String("plains".to_owned()));

    let mut effects = nbt::Map::new();
    const colors: &[&str] = &["sky_color", "water_fog_color", "fog_color", "water_color"];
    for &color in colors {
        effects.insert(color.to_owned(), Int(8364534i32));
    }
    let effects = nbt::Value::Compound(effects);
    default.insert("effects".to_owned(), effects);
    nbt::Value::Compound(default)
}

fn default_biome_entry() -> nbt::Value {
    registry_entry_generic("minecraft:plains", 0, default_biome_properties())
}

fn default_biome_registry() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert(
        "type".to_owned(),
        String("minecraft:worldgen/biome".to_owned()),
    );
    // let biome_list = Vec::new();
    // default.insert("value".to_owned(), List(biome_list));
    let mut biome_map = nbt::Map::new();
    biome_map.insert("minecraft:default".to_owned(), default_biome_entry());
    default.insert("value".to_owned(), Compound(biome_map));
    nbt::Value::Compound(default)
}

fn default_dimension_codec() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert(
        "minecraft:dimension_type".to_owned(),
        default_dimension_registry(),
    );
    default.insert(
        "minecraft:worldgen/biome".to_owned(),
        default_biome_registry(),
    );
    nbt::Value::Compound(default)
}

lazy_static! {
    pub static ref DEFAULT_DIMENSION_TYPE: nbt::Value = default_dimension_type();
    pub static ref DEFAULT_DIMENSION_CODEC: nbt::Value = default_dimension_codec();
}
