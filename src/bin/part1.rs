use lamina::*;
use lamina::ctx::PMCContext;
use lamina::pmc::PerfCtlDescriptor;
use lamina::event::Event;
use lamina::x86::*;

// A macro for assembling a test
macro_rules! emit_branch_test {
    ($ptr:ident $($body:tt)*) => { {

        // 'lamina' wrapper macro for RDPMC measurements.
        // Most general-purpose registers have been cleared.
        // The PMCs are enabled and counting events inside this block.
        emit_rdpmc_test_all!(

            // A pointer passed to the PREFETCH instruction
            ; mov rdi, QWORD $ptr as _

            // Compute the effective address of these two labels that
            // we expect to define in the body of our test.
            // This will make testing indirect branches a little easier.
            ; lea rsi, [->branch_target]
            ; lea rbx, [->end]

            // Clear RAX [and set the zero flag].
            ; xor rax, rax

            // Stop any speculative paths from entering our test code
            ; lfence

            // The body of our test is inserted here at compile-time
            $($body)*

            // Stop any speculative paths from escaping our test code
            ; lfence
        )
    } }
}


/// Wrapper function for running a particular test.
fn run_test(ctx: &mut PMCContext, e: &PerfCtlDescriptor, 
            code: &ExecutableBuffer, desc: &'static str, 
            filename: &'static str) 
    -> Result<(), &'static str> 
{
    // Set up the set of events we're going to use
    ctx.write(&e)?;

    // Run the test some number of times
    let mut test = PMCTest::new(&desc, &code, &e);
    test.run_iter(4096);
    test.print();
    println!("");

    // Write the results to a plaintext file
    test.res.write_txt(filename);
    Ok(())
}


fn main() -> Result<(), &'static str> {
    lamina::util::pin_to_core(0);

    let mut ctx = PMCContext::new()?;
    let scratch = Box::new([0x08u8; 256]);
    let scratch_ptr = scratch.as_ptr();

    let events = PerfCtlDescriptor::new()
        .set(0, Event::LsPrefInstrDisp(0x01))
        .set(1, Event::ExRetBrnMisp(0x00))
        .set(2, Event::ExRetBrn(0x00));

    // An always-taken conditional branch.
    let code = emit_branch_test!(scratch_ptr
        ; jz ->branch_target

        ; prefetch [rdi]
        ; lfence

        ; .align 64
        ; ->branch_target:
        ; ->end:
    );
    run_test(&mut ctx, &events, &code, "jcc (always-taken)", "/tmp/jcc_always.txt")?;

    // A never-taken conditional branch.
    let code = emit_branch_test!(scratch_ptr
        ; jnz ->branch_target

        ; .align 64
        ; jmp ->end
        ; lfence

        ; .align 64
        ; ->branch_target:
        ; prefetch [rdi]
        ; lfence

        ; .align 64
        ; ->end:
    );
    run_test(&mut ctx, &events, &code, "jcc (never-taken)", "/tmp/jcc_never.txt")?;

    Ok(())
}

