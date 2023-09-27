pub enum Value {
    None,
    Integer(i64),
    Float(f64),
    String(String),
}

pub enum Type {
    Bool,
    Int,
    Uint,
    Dword,
    Half,
    Float,
    Double,
    Min16Float,
    Min10Float,
    Min16Int,
    Min12Int,
    Min16Uint,
    Uint64t,
    Int64t,
    Float16t,
    Uint16t,
    Int16t,
    String,
    Matrix,
    Vector
}