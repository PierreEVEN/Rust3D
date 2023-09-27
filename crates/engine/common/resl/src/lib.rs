mod ast;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub language); // synthesized by LALRPOP

fn get_error_location(code: &str, token: usize) -> (usize, usize) {
    let mut line = 0;
    let mut token_to_last_line = 0;
    let mut elapsed_tokens = 0;
    for chr in code.chars() {
        if chr == '\n' {
            line += 1;
            token_to_last_line = elapsed_tokens;
        }
        elapsed_tokens += 1;
        if elapsed_tokens >= token {
            break;
        }
    }
    (line, token - token_to_last_line)
}

#[test]
fn calculator1() {

    let code = r#"
    #pragma once twice
    #pragma test two
    #pragma ENABLE_HLSL
    #pragma integer 4
    #pragma version "4.5.8.9 test"

    global(test, "shader a", shader_b) {
        struct VSInput
        {
            float2 aPos 	: POSITION;
            float2 aUV 		: TEXCOORD;
            float4 aColor 	: COLOR;
        }

        struct PushConsts
        {
            float2 uScale;
            float2 uTranslate;
        }

        VsToFs main(VSInput input)
        {
            VsToFs Out;
            Out.Color	= input.aColor;
            Out.UV 		= input.aUV;
            Out.Pos 	= float4(input.aPos * pc.uScale + pc.uTranslate, 0, 1);
            return Out;
        }
    }

    vertex(example) {
        // declaration of functions

        // data structure : before vertex shader (mesh info)
        struct vertexInfo
        {
            float3 position : POSITION;
            float2 uv: TEXCOORD0;
            float3 color : COLOR;
        }

        // data structure : vertex shader to pixel shader
        // also called interpolants because values interpolates through the triangle
        // from one vertex to another
        struct v2p
        {
            float4 position : SV_POSITION;
            float3 uv : TEXCOORD0;
            float3 color : TEXCOORD1;
        }

        // uniforms : external parameters
        sampler2D MyTexture; //@TODO : property are not supported
        float2 UVTile;
        matrix4x4 worldViewProjection;

        // vertex shader function
        v2p vertexShader(vertexInfo input)
        {
            v2p output;
            output.position = mul(worldViewProjection, float4(input.position,1.0));
            output.uv = input.uv * UVTile;
            output.color = input.color;
            return output;
        }

        // pixel shader function
        float4 pixelShader(v2p input) : SV_TARGET
        {
            float4 color = tex2D(MyTexture, input.uv);
            return color * input.color;
        }
    }

    "#;

    let parsed = match language::InstructionListParser::new().parse(code) {
        Ok(res) => res,
        Err(e) => match e {
            lalrpop_util::ParseError::UnrecognizedToken {token, expected} => {
                let (a, b, c) = token;
                // Comment lire le contenu de UnrecognizedToken ?
                let (line, column) = get_error_location(code, a);

                panic!("Unrecognized token at {}:{} : Found {:?}\nexpected {:?}", line, column, b, expected)
            },
            lalrpop_util::ParseError::InvalidToken {location} => {
                let (line, column) = get_error_location(code, location);
                panic!("Unrecognized token at {}:{} : '{:?}'", line, column, code.chars().nth(location).unwrap())

            },
            _ => {
                panic!("Compilation failed:\n{:?}", e)
            }
        }
    };

    println!("${:?}", parsed);
}