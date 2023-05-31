mod infra;

// Your tests go here!
success_tests! {
    {
        name: make_vec_succ,
        file: "make_vec.snek",
        input: "5",
        expected: "[0, 0, 0, 0, 0]",
    },
    {
        name: vec_succ,
        file: "vec.snek",
        expected: "[0, 1, 2, 3]",
    },
    {
        name: vec_get_succ,
        file: "vec_get.snek",
        input: "3",
        expected: "3",
    },
    {
        name: linked_list_manipulations,
        file: "linked_list_manipulations.snek",
        expected: "1\n2\n3\n4\n5\n5\n4\n3\n2\n1\nnil"
    },
    {
        name: bst_pass,
        file: "bst.snek",
        heap_size: 50,
        expected: "true",
    },
    {
        name: empty_heap_gc,
        file: "empty_heap_gc.snek",
        expected: "0",
    },
    {
        name: no_heap_gc,
        file: "empty_heap_gc.snek",
        input: "0",
        heap_size: 0,
        expected: "0",
    },
    {
        name: bst_loop_1,
        file: "bst_loop.snek",
        input: "10",
        heap_size: 500,
        expected: "[1, false, [2, false, [3, false, [4, false, [5, false, [6, false, [7, false, [8, false, [9, false, [10, false, false]]]]]]]]]]",
    },
    {
        name: bst_loop_2,
        file: "bst_loop.snek",
        input: "10",
        heap_size: 300,
        expected: "[1, false, [2, false, [3, false, [4, false, [5, false, [6, false, [7, false, [8, false, [9, false, [10, false, false]]]]]]]]]]",
    },
    {
        name: bst_loop_3,
        file: "bst_loop.snek",
        input: "10",
        heap_size: 200,
        expected: "[1, false, [2, false, [3, false, [4, false, [5, false, [6, false, [7, false, [8, false, [9, false, [10, false, false]]]]]]]]]]",
    },
    {
        name: set_gc_set,
        file: "set_gc_set.snek",
        heap_size: 5,
        expected: "[4, 5, 6]",
    },
    {
        name: zeros_vec,
        file: "zeros_vec.snek",
        expected: "[1, 1, 1]\n0",
    },
    {
        name: simple_gc,
        file: "simple_garbage.snek",
        expected: "0",
    },

}

runtime_error_tests! {
    {
        name: make_vec_oom,
        file: "make_vec.snek",
        input: "5",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: vec_get_oob,
        file: "vec_get.snek",
        input: "5",
        expected: "",
    },
    {
        name: bst_run_out_mem,
        file: "bst.snek",
        input: "5",
        heap_size: 5,
        expected: "out of memory",
    },
    {
        name: bst_loop_4,
        file: "bst_loop.snek",
        input: "10",
        heap_size: 50,
        expected: "out of memory",
    },

}

static_error_tests! {}
