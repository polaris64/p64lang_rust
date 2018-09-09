const js = import('./p64lang_wasm');

function get_source() {
  return document.getElementById('txt_source');
}

function get_stdout() {
  return document.getElementById('stdout');
}

export function js_print(s, nl) {
  const div_stdout = get_stdout();
  div_stdout.innerHTML = div_stdout.innerHTML + s + (nl ? "<br />" : '');
}

js.then((js) => {

  const btn_run = document.getElementById('btn_run');
  const btn_clr = document.getElementById('btn_clr');

  btn_run.addEventListener('click', () => {
    js_print(js.interpret_str(get_source().value), true);
  });

  btn_clr.addEventListener('click', () => {
    get_stdout().innerHTML = '';
  });

  btn_load1.addEventListener('click', () => {
    get_source().value = 'fn fib(n) {\n\
    if n <= 1 { return n; };\n\
    return fib(n - 2) + fib(n - 1);\n\
};\n\n\
let counter = 0;\n\
loop {\n\
    println("fib(", counter, ") = ", fib(counter));\n\
    let counter = counter + 1;\n\
    if counter > 20 { break; };\n\
}';
  });

  btn_load2.addEventListener('click', () => {
    get_source().value = 'fn fib(n) {\n\
  if n < 1 { return 0; };\n\
  let res = 0;\n\
  let prev = 1;\n\
  let i = n;\n\
  loop {\n\
    let temp = res;\n\
    let res = res + prev;\n\
    let prev = temp;\n\
    let i = i - 1;\n\
    if i == 0 { break; };\n\
  };\n\
  return res;\n\
};\n\n\
let counter = 0;\n\
loop {\n\
  println("fib(", counter, ") = ", fib(counter));\n\
  let counter = counter + 1;\n\
  if counter > 28 { break; };\n\
}';
  });

  btn_load3.addEventListener('click', () => {
    get_source().value = 'let fib_cache = [];\n\n\
fn fib(n) {\n\
  if n <= 1 { return n; };\n\
  let a = fib_cache[n - 1];\n\
  let b = fib_cache[n - 2];\n\
  if !a {\n\
    fib_cache[n - 1] = fib(n - 1);\n\
    let a = fib_cache[n - 1];\n\
  };\n\
  if !b {\n\
    fib_cache[n - 2] = fib(n - 2);\n\
    let b = fib_cache[n - 2];\n\
  };\n\
  return a + b;\n\
};\n\n\
let counter = 0;\n\
loop {\n\
  println("fib(", counter, ") = ", fib(counter));\n\
  let counter = counter + 1;\n\
  if counter > 28 { break; };\n\
}';
  });
});
