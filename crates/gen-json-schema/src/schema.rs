use valico::json_schema::{builder::Dependencies, PrimitiveType};
use PrimitiveType::{Integer, Number};

pub fn add_primitives(deps: &mut Dependencies) {
    deps.schema("u64", |b| {
        b.type_(Integer);
        b.minimum(0f64);
        b.maximum(18446744073709551615f64);
    });
    deps.schema("i64", |b| {
        b.type_(Integer);
        b.minimum(-9223372036854775808f64);
        b.maximum(9223372036854775807f64);
    });
    deps.schema("u8", |b| {
        b.type_(Integer);
        b.minimum(0f64);
        b.maximum(255f64);
    });
    deps.schema("i8", |b| {
        b.type_(Integer);
        b.minimum(-128f64);
        b.maximum(127f64);
    });
    deps.schema("u16", |b| {
        b.type_(Integer);
        b.minimum(0f64);
        b.maximum(65535f64);
    });
    deps.schema("i16", |b| {
        b.type_(Integer);
        b.minimum(-32768f64);
        b.maximum(32767f64);
    });
    deps.schema("u32", |b| {
        b.type_(Integer);
        b.minimum(0f64);
        b.maximum(4294967295f64);
    });
    deps.schema("usize", |b| {
        b.type_(Integer);
        b.minimum(0f64);
        b.maximum(4294967295f64);
    });
    deps.schema("i32", |b| {
        b.type_(Integer);
        b.minimum(-2147483648f64);
        b.maximum(2147483647f64);
    });

    deps.schema("f32", |b| {
        b.type_(Number);
        b.minimum(-3.40282347E+38);
        b.maximum(3.40282347E+38);
    });

    deps.schema("f64", |b| {
        b.type_(Number);
        b.minimum(-1.797_693_134_862_315_7E308);
        b.maximum(1.797_693_134_862_315_7E308);
    });

    deps.schema("unit", |_b| {});

    deps.schema("char", |b| {
        b.string();
        b.max_length(1);
    });
}

/*
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "$ref": "#/definitions/Balance",
  "definitions": {
    "Balance": {
      "$ref": "#/definitions/U128",
      "description": "Balance is a type for storing amounts of tokens, specified in yoctoNEAR."
    },
    "CallOptions": {
      "additionalProperties": false,
      "properties": {
        "attachedDeposit": {
          "$ref": "#/definitions/Balance",
          "default": "0",
          "description": "Units in yoctoNear"
        },
        "gas": {
          "default": "30000000000000",
          "description": "Units in gas",
          "pattern": "[0-9]+",
          "type": "string"
        }
      },
      "type": "object"
    },
    "CurrentVersion": {
      "additionalProperties": false,
      "contractMethod": "view",
      "properties": {
        "args": {
          "additionalProperties": false,
          "type": "object"
        }
      },
      "required": [
        "args"
      ],
      "type": "object"
    },
    "CurrentVersion__Result": {
      "type": "string"
    },
    "Fetch": {
      "additionalProperties": false,
      "contractMethod": "view",
      "properties": {
        "args": {
          "additionalProperties": false,
          "type": "object"
        }
      },
      "required": [
        "args"
      ],
      "type": "object"
    },
    "Fetch__Result": {
      "type": "null"
    },
    "Major": {
      "additionalProperties": false,
      "contractMethod": "change",
      "description": "Breaking change",
      "properties": {
        "args": {
          "additionalProperties": false,
          "type": "object"
        },
        "options": {
          "$ref": "#/definitions/CallOptions"
        }
      },
      "required": [
        "args",
        "options"
      ],
      "type": "object"
    },
    "Major__Result": {
      "type": "null"
    },
    "Minor": {
      "additionalProperties": false,
      "contractMethod": "change",
      "description": "Non-breaking feature",
      "properties": {
        "args": {
          "additionalProperties": false,
          "type": "object"
        },
        "options": {
          "$ref": "#/definitions/CallOptions"
        }
      },
      "required": [
        "args",
        "options"
      ],
      "type": "object"
    },
    "Minor__Result": {
      "type": "null"
    },
    "Patch": {
      "additionalProperties": false,
      "contractMethod": "change",
      "description": "Non-breaking fix",
      "properties": {
        "args": {
          "additionalProperties": false,
          "type": "object"
        },
        "options": {
          "$ref": "#/definitions/CallOptions"
        }
      },
      "required": [
        "args",
        "options"
      ],
      "type": "object"
    },
    "Patch__Result": {
      "type": "null"
    },
    "U128": {
      "description": "String representation of a u128-bit integer",
      "pattern": "^[0-9]+$",
      "type": "string"
    }
  }
}
*/
