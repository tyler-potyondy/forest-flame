use std::{collections::HashSet, env};

type SnekVal = u64;

#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum ErrCode {
    InvalidArgument = 1,
    Overflow = 2,
    IndexOutOfBounds = 3,
    InvalidVecSize = 4,
    OutOfMemory = 5,
}

const TRUE: u64 = 7;
const FALSE: u64 = 3;

static mut HEAP_START: *const u64 = std::ptr::null();
static mut HEAP_END: *const u64 = std::ptr::null();

#[link(name = "our_code")]
extern "C" {
    // The \x01 here is an undocumented feature of LLVM that ensures
    // it does not add an underscore in front of the name.
    // Courtesy of Max New (https://maxsnew.com/teaching/eecs-483-fa22/hw_adder_assignment.html)
    #[link_name = "\x01our_code_starts_here"]
    fn our_code_starts_here(input: u64, heap_start: *const u64, heap_end: *const u64) -> u64;
}

#[export_name = "\x01snek_error"]
pub extern "C" fn snek_error(errcode: i64) {
    if errcode == ErrCode::InvalidArgument as i64 {
        eprintln!("invalid argument");
    } else if errcode == ErrCode::Overflow as i64 {
        eprintln!("overflow");
    } else if errcode == ErrCode::IndexOutOfBounds as i64 {
        eprintln!("index out of bounds");
    } else if errcode == ErrCode::InvalidVecSize as i64 {
        eprintln!("vector size must be non-negative");
    } else {
        eprintln!("an error ocurred {}", errcode);
    }
    std::process::exit(errcode as i32);
}

#[export_name = "\x01snek_print"]
pub unsafe extern "C" fn snek_print(val: SnekVal) -> SnekVal {
    println!("{}", snek_str(val, &mut HashSet::new()));
    val
}

/// This function is called when the program needs to allocate `count` words of memory and there's no
/// space left. The function should try to clean up space by triggering a garbage collection. If there's
/// not enough space to hold `count` words after running the garbage collector, the program should terminate
/// with an `out of memory` error.
///
/// Args:
///     * `count`: The number of words the program is trying to allocate, including an extra word for
///       the size of the vector and an extra word to store metadata for the garbage collector, e.g.,
///       to allocate a vector of size 5, `count` will be 7.
///     * `heap_ptr`: The current position of the heap pointer (i.e., the value stored in `%r15`). It
///       is guaranteed that `heap_ptr + 8 * count > HEAP_END`, i.e., this function is only called if
///       there's not enough space to allocate `count` words.
///     * `stack_base`: A pointer to the "base" of the stack.
///     * `curr_rbp`: The value of `%rbp` in the stack frame that triggered the allocation.
///     * `curr_rsp`: The value of `%rsp` in the stack frame that triggered the allocation.
///
/// Returns:
///
/// The new heap pointer where the program should allocate the vector (i.e., the new value of `%r15`)
///
#[export_name = "\x01snek_try_gc"]
pub unsafe fn snek_try_gc(
    count: isize,
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    let new_heap_ptr = snek_gc(
        heap_ptr,
        stack_base,
        curr_rbp,
        curr_rsp,
    );

    // println!("HEAP PTR {:?}", heap_ptr);
    // println!("NEW HEAP PTR {:?}", new_heap_ptr);
    // println!("freed words : {}",(heap_ptr as u64 - new_heap_ptr as u64)/8);
    // println!("REQUESTED WORDS : {count}");
    // println!("INFO {:#0x}",(new_heap_ptr as u64 + 8 as u64) * count as u64);
    // println!("HEAP END --> {:?}",HEAP_END);
    // print_heap(heap_ptr);

    if (new_heap_ptr as u64) + (8 * count) as u64 > HEAP_END as u64 {
        eprintln!("out of memory");
        std::process::exit(ErrCode::OutOfMemory as i32)
    }

    new_heap_ptr
}

unsafe fn find_stack_roots(
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
)-> Vec<*mut u64> {
    let mut ptr = stack_base;
    let mut stack_roots = Vec::new();
    while ptr >= curr_rsp {
        let val = unsafe {*ptr};
        if is_heap_obj(val) {
            stack_roots.push(ptr as *mut u64);
        }
        ptr = unsafe {ptr.sub(1)};
    }
    
    stack_roots
}

fn mark(roots: Vec<*mut u64>) {
    for item in roots {
        unsafe {println!("ROOT HEAP ADDR {:#0x}", (*item - 1))};
        unsafe {heap_mark((*item-1) as *mut u64)};
    }
}

unsafe fn is_heap_obj(obj: u64) -> bool {
    if obj == TRUE || obj == FALSE || obj == 1 || obj & 1 == 0{
        return false
    }
    if obj & 1 == 1 && obj <= (HEAP_END as u64) && obj >= (HEAP_START as u64) {
        return true
    } 
    false 
}

unsafe fn heap_mark(obj_addr: *mut u64) {
    ///////////////////
    // obj_addr is the heap address of a heap object
    ///////////////////

    // mark this heap object
    *obj_addr = 1;

    // iterate through remaining items stored in heap object to determine 
    // if there exists pointer to other heap object, recursively mark these items
    let obj_len = obj_addr.add(1).read() as usize;
    let mut ind = 0;

    println!("OBJ UPDATE --> {:#0x}",*obj_addr);
    println!("next len {obj_len}");
    while ind < obj_len {
        let heap_val = *obj_addr.add(2+ind);
        if is_heap_obj(heap_val) {
            println!("AA {}",heap_val);
            heap_mark((heap_val -1) as *mut u64);
        }
        ind+=1;
    }

    println!("completed --> ")

}

unsafe fn fwd_headers(heap_ptr: *const u64){
    let mut from = HEAP_START as *mut u64;
    let mut to = HEAP_START as *mut u64;

    while from < heap_ptr as *mut u64 {
        if (*from) == 1 {
            *from = (to as u64) + 1;
            to = to.add((2+*to.add(1)) as usize);
            from = from.add((2+*from.add(1)) as usize);

        } else if (*from) == 0 {
            from = from.add((2+*from.add(1)) as usize);

        } else {
            panic!("Misalignment has occured during GC.")
        }
    } 
}

unsafe fn fwd_internal(roots: Vec<*mut u64>){
    for stack_ref in roots {
        let heap_obj = (*stack_ref - 1) as *mut u64;
        fwd_heap(heap_obj);
        update_stack(stack_ref);
    }
}

/// Update references on the stack
unsafe fn update_stack(stack_ref: *mut u64){
    let heap_addr = (*stack_ref - 1) as *mut u64;
    let heap_val  = *heap_addr;
    *stack_ref = heap_val + 1;
}

/// Update internal heap references
unsafe fn fwd_heap(obj: *mut u64){
    if *obj & 1 == 0 {
        return
    }
    *obj = *obj-1; // mark as forwarded

    let obj_len = (obj).add(1).read() as usize;
    let mut ind = 0;

    while ind < obj_len {
        let heap_val = *obj.add(2+ind);
        if is_heap_obj(heap_val) {
            let mut fwd_addr = *((heap_val-1)as *mut u64);
            if fwd_addr & 1 == 1 {
                fwd_addr-= 1;
            }
            let mut obj_ref = obj.add(2+ind);
            *obj_ref = fwd_addr;
            fwd_heap((heap_val-1) as *mut u64)
        }
        ind+=1;
    }
}

/// Iterate through heap compacting references and resetting mark word
unsafe fn compact(heap_ptr: *const u64) -> u64 {
    // print_heap(heap_ptr);
    let mut addr = HEAP_START as *mut u64;

    let mut remain_garb = 0;
    let mut total_garb = 0;

    // first pass to mark all garbage to zero
    while addr < heap_ptr as *mut u64 {
        // find garbage memory and length of garbage 
        if (*addr) == 0 {
            println!("Found garb at addr {:?}",addr);
            remain_garb = (addr.add(1).read() + 2) as usize;
            total_garb += remain_garb;
            // advance address to end of garbage memory (next heap object)
            addr = addr.add(remain_garb);
        
            let mut temp_addr = addr;
            println!("SHIFT START {:?}",temp_addr);
            // shift every word after this down by the length of garbage memory
            while temp_addr < heap_ptr as *mut u64 {
                let mut garb_mem = temp_addr.sub(remain_garb);
                *garb_mem = *temp_addr;
                temp_addr = temp_addr.add(1);
            }
        } else {
            addr = addr.add((addr.add(1).read() + 1) as usize) as *mut u64
        }
    }

    // hop through heap unmarking all metabit used for garbage collection
    addr = HEAP_START as *mut u64;
    while addr < heap_ptr.sub(total_garb + 1) as *mut u64 {
        let obj_len = (addr.add(1).read() + 2) as usize;
        *addr = 0;
        addr = addr.add(obj_len);
    }

    // println!("FW {total_garb}");
    return total_garb as u64
}

/// This function should trigger garbage collection and return the updated heap pointer (i.e., the new
/// value of `%r15`). See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_gc"]
pub unsafe fn snek_gc(
    heap_ptr: *const u64,
    stack_base: *const u64,
    curr_rbp: *const u64,
    curr_rsp: *const u64,
) -> *const u64 {
    print_heap(heap_ptr);
    snek_print_stack(stack_base,curr_rbp,curr_rsp);

    // first find all roots on the stack (i.e. search for anything with heap data tag)
    let roots = find_stack_roots(stack_base,curr_rbp,curr_rsp);

    println!("Found roots:: {:?}",roots);
    // mark active heap objects
    mark(roots.clone());

    // forward headers of marked objects
    fwd_headers(heap_ptr);

    // forward internal references and stack references
    fwd_internal(roots.clone());
    
    // print_heap(heap_ptr);
    // compact heap
    let removed_words = compact(heap_ptr);

    // println!("///FINAL HEAP:");
    // print_heap(heap_ptr.sub(removed_words as usize));

    heap_ptr.sub(removed_words as usize)

}


/// Helper function to print heap
unsafe fn print_heap(heap_ptr: *const u64) {
    let mut ptr = HEAP_START;
    println!("************************");
    while ptr < heap_ptr {
        let val = *ptr;
        // if val != 0 {
            println!("{ptr:?}: {:#0x}", val);
        // }
        ptr = ptr.add(1);
    }
    println!("************************");

}

/// A helper function that can called with the `(snek-printstack)` snek function. It prints the stack
/// See [`snek_try_gc`] for a description of the meaning of the arguments.
#[export_name = "\x01snek_print_stack"]
pub unsafe fn snek_print_stack(stack_base: *const u64, curr_rbp: *const u64, curr_rsp: *const u64) {
    let mut ptr = stack_base;
    println!("-----------------------------------------");
    while ptr >= curr_rsp {
        let val = *ptr;
        println!("{ptr:?}: {:#0x}", val);
        ptr = ptr.sub(1);
    }
    println!("-----------------------------------------");
}

unsafe fn snek_str(val: SnekVal, seen: &mut HashSet<SnekVal>) -> String {
    if val == TRUE {
        format!("true")
    } else if val == FALSE {
        format!("false")
    } else if val & 1 == 0 {
        format!("{}", (val as i64) >> 1)
    } else if val == 1 {
        format!("nil")
    } else if val & 1 == 1 {
        if !seen.insert(val) {
            return "[...]".to_string();
        }
        let addr = (val - 1) as *const u64;
        let size = addr.add(1).read() as usize;
        let mut res = "[".to_string();
        for i in 0..size {
            let elem = addr.add(2 + i).read();
            res = res + &snek_str(elem, seen);
            if i < size - 1 {
                res = res + ", ";
            }
        }
        seen.remove(&val);
        res + "]"
    } else {
        format!("unknown value: {val}")
    }
}

fn parse_input(input: &str) -> u64 {
    match input {
        "true" => TRUE,
        "false" => FALSE,
        _ => (input.parse::<i64>().unwrap() << 1) as u64,
    }
}

fn parse_heap_size(input: &str) -> usize {
    input.parse::<usize>().unwrap()
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let input = if args.len() >= 2 { &args[1] } else { "false" };
    let heap_size = if args.len() >= 3 { &args[2] } else { "10000" };
    let input = parse_input(&input);
    let heap_size = parse_heap_size(&heap_size);

    // Initialize heap
    let mut heap: Vec<u64> = Vec::with_capacity(heap_size);
    unsafe {
        HEAP_START = heap.as_mut_ptr();
        HEAP_END = HEAP_START.add(heap_size);
    }

    let i: u64 = unsafe { our_code_starts_here(input, HEAP_START, HEAP_END) };
    unsafe { snek_print(i) };
}
