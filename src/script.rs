use anyhow::{Context, Result};
use jsonpath_lib as jsonpath;
use rhai::{Dynamic, Engine, EvalAltResult, ImmutableString, Map as RhaiMap, Scope};
use serde_json::{Map as JsonMap, Value};
use std::{collections::HashMap, fs};

pub struct ScriptEnv<'a> {
    pub vars: &'a mut HashMap<String, String>,
    pub status: Option<i64>,
    pub headers: Option<HashMap<String, String>>,
    pub data: Option<Value>,
    pub project: String,
    pub env: String,
}
fn engine() -> Engine {
    let mut eng = Engine::new();
    eng.register_fn(
        "expect_toEqual",
        |a: Dynamic, b: Dynamic| -> Result<(), Box<EvalAltResult>> {
            let va: serde_json::Value = rhai::serde::from_dynamic(&a)
                .map_err(|e| format!("expect_toEqual: cannot convert lhs: {e}"))?;
            let vb: serde_json::Value = rhai::serde::from_dynamic(&b)
                .map_err(|e| format!("expect_toEqual: cannot convert rhs: {e}"))?;
            if va == vb {
                Ok(())
            } else {
                Err(format!("Assertion failed: {va:?} != {vb:?}").into())
            }
        },
    );
    eng.register_fn(
        "expect_toContain",
        |s: Dynamic, sub: String| -> Result<(), Box<EvalAltResult>> {
            let text = if let Some(ss) = s.clone().try_cast::<String>() {
                ss
            } else {
                let v: serde_json::Value = rhai::serde::from_dynamic(&s)
                    .map_err(|e| format!("expect_toContain: cannot convert value: {e}"))?;
                v.to_string()
            };
            if text.contains(&sub) {
                Ok(())
            } else {
                Err(format!("Assertion failed: '{text}' does not contain '{sub}'").into())
            }
        },
    );
    eng.register_fn("jsonPath", |data: Dynamic, expr: String| -> Dynamic {
        if let Some(v) = data.try_cast::<serde_json::Value>() {
            match jsonpath::select(&v, &expr) {
                Ok(nodes) if !nodes.is_empty() => {
                    rhai::serde::to_dynamic(nodes[0].clone()).unwrap_or(Dynamic::UNIT)
                }
                _ => Dynamic::UNIT,
            }
        } else {
            Dynamic::UNIT
        }
    });

    eng
}
pub fn run_script(code: &str, senv: &mut ScriptEnv) -> Result<()> {
    let eng = engine();
    let mut scope = Scope::new();

    if let Some(status) = senv.status {
        scope.push("status", status);
    }
    if let Some(ref headers) = senv.headers {
        scope.push("headers", headers.clone());
    }
    if let Some(ref data) = senv.data {
        scope.push_dynamic("data", rhai::serde::to_dynamic(data.clone())?);
    }

    let mut env_map: RhaiMap = RhaiMap::new();
    for (k, v) in senv.vars.iter() {
        env_map.insert(
            ImmutableString::from(k.as_str()).into(),
            Dynamic::from(v.clone()),
        );
    }
    scope.push_dynamic("env", Dynamic::from(env_map));

    let _ = eng
        .eval_with_scope::<Dynamic>(&mut scope, code)
        .map_err(|e| anyhow::anyhow!("Rhai error: {e}"))?;

    if let Some(updated_env) = scope.get_value::<RhaiMap>("env") {
        senv.vars.clear();
        for (k, v) in updated_env.into_iter() {
            if let Some(s) = v.clone().try_cast::<String>() {
                senv.vars.insert(k.to_string(), s);
            } else {
                senv.vars.insert(k.to_string(), v.to_string());
            }
        }
        crate::db::upsert_vars(&senv.project, &senv.env, senv.vars)?;
    }

    Ok(())
}
