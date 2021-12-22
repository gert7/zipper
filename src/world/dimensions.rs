use lazy_static::lazy_static;
use nbt::Value::*;

const NBT_ITRUE: nbt::Value = Byte(1i8);
const NBT_IFALSE: nbt::Value = Byte(0i8);

fn registry_entry_generic(name: &str, id: i32, element: nbt::Value) -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert("name".to_owned(), String(name.to_owned()));
    default.insert("id".to_owned(), Int(id));
    default.insert("element".to_owned(), element);
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
    let mut dimension_list = vec![default_dimension_entry()];
    default.insert("value".to_owned(), List(dimension_list));
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
    registry_entry_generic("minecraft:default", 0, default_biome_properties())
}

fn default_biome_registry() -> nbt::Value {
    let mut default = nbt::Map::new();
    default.insert(
        "type".to_owned(),
        String("minecraft:worldgen/biome".to_owned()),
    );
    let biome_list = vec![default_biome_entry()];
    default.insert("value".to_owned(), List(biome_list));
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
