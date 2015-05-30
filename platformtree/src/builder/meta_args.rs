// Zinc, the bare metal stack for rust.
// Copyright 2014 Vladimir "farcaller" Pouzanov <farcaller@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use syntax::ast;
use syntax::codemap::DUMMY_SP;
use syntax::ext::base::ExtCtxt;
use syntax::ext::build::AstBuilder;
use syntax::parse::token::{InternedString, intern_and_get_ident};
use syntax::ptr::P;
use std::hash::{hash, SipHasher};

static TAG: &'static str = "__zinc_task_ty_params";

pub trait ToTyHash {
  fn to_tyhash(&self) -> String;
}

impl ToTyHash for String {
  fn to_tyhash(&self) -> String {
    let h: u64 = hash::<_, SipHasher>(&self);
    format!("Ty{:X}", h)
  }
}

/// Sets ty_params for named task.
///
/// Arguments:
///   task: task name. This function must be called exactly once per task
///   args: a vector of type parameters
pub fn set_ty_params_for_task(cx: &mut ExtCtxt, task: &str, args: Vec<String>) {
  let ty_params = args.iter().map(|arg| {
    cx.meta_word(DUMMY_SP, intern_and_get_ident(arg.as_str()))
  }).collect();
  let newmi = cx.meta_list(DUMMY_SP, intern_and_get_ident(task), ty_params);

  let mut tasks = get_tasks(cx);
  tasks.push(newmi);

  set_tasks(cx, tasks);
}

/// Returns a vector of type parameters for task.
pub fn get_ty_params_for_task(cx: &ExtCtxt, task: &str) -> Vec<String> {
  get_task(&get_tasks(cx), task)
}

/// Inserts or replaces tasks vector
fn set_tasks(cx: &mut ExtCtxt, tasks: Vec<P<ast::MetaItem>>) {
  let mut vec_clone = cx.cfg();
  let maybe_pos = vec_clone.iter().position(|i| {
    match i.node {
      ast::MetaList(ref k, _) if *k == TAG => true,
      _ => false,
    }
  });

  if maybe_pos.is_some() {
    vec_clone.remove(maybe_pos.unwrap());
  }
  vec_clone.push(cx.meta_list(DUMMY_SP, InternedString::new(TAG), tasks));

  cx.cfg = vec_clone;
}

/// Returns a vector of MetaLists where each MetaList corresponds to one task.
fn get_tasks(cx: &ExtCtxt) -> Vec<P<ast::MetaItem>> {
  for i in cx.cfg.iter() {
    match i.node {
      ast::MetaList(ref k, ref v) if *k == TAG => return v.clone(),
      _ => (),
    }
  };
  vec!()
}

/// Returns a vector of type parameters for named task.
fn get_task(tasks: &Vec<P<ast::MetaItem>>, task: &str) -> Vec<String> {
  let mut ty_params = vec!();
  for mi in tasks.iter() {
    match mi.node {
      ast::MetaList(ref k, ref v) if *k == task => {
        for submi in v.iter() {
          match submi.node {
            ast::MetaWord(ref w) => ty_params.push((&*w).to_string()),
            _ => panic!("unexpected node type"),
          }
        }
        break;
      },
      _ => (),
    }
  }
  ty_params
}

// pub fn get_ty_params_for_task(cx: &ExtCtxt, task: &str) -> Option<Vec<String>> {
//   get_task(cx, task).and_then(|ma| Some(ma.extra_ty_params.clone()))
// }

// fn get_task(cx: &ExtCtxt, task: &str) -> Option<MetaArgs> {
//   get_args(cx).and_then(|args| {
//     for a in args.iter() {
//       if a.task_name.as_str() == task {
//         return Some(a.clone());
//       }
//     }
//     None
//   })
// }

// fn get_args(cx: &ExtCtxt) -> Option<Vec<MetaArgs>> {
//   cx.cfg.iter().find(|i| {
//     match i.node {
//       ast::MetaList(ref k, _) => k.get() == TAG,
//       _ => false,
//     }
//   }).and_then(|i| match i.node {
//     ast::MetaList(_, ref v) => Some(meta_item_to_meta_args(v)),
//     _ => panic!(),
//   })
// }

// fn meta_item_to_meta_args(mi: &Vec<P<ast::MetaItem>>) -> Vec<MetaArgs> {
//   let mut args = vec!();
//   for i in mi.iter() {
//     match i.node {
//       ast::MetaWord(ref istr) => {
//         let s = istr.get();

//         args.push(json::decode(s).unwrap());
//       },
//       _ => panic!(),
//     }
//   }

//   args
// }

// fn meta_args_to_meta_item(name: String, args: Vec<String>) -> P<ast::MetaItem> {
//   let ma = MetaArgs {
//     task_name: name,
//     extra_ty_params: args,
//   };
//   let enc = json::encode(&ma);
//   let istr = intern_and_get_ident(enc.as_str());
//   box(GC) respan(DUMMY_SP, ast::MetaWord(istr))
// }
