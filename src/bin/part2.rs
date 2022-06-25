use lamina::*;
use lamina::ctx::PMCContext;
use lamina::pmc::PerfCtlDescriptor;
use lamina::event::Event;
use lamina::x86::*;

macro_rules! emit_branch_test_aligned {
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

            // Pad LFENCE to the last three bytes in the cache line.
            // This means our test code starts at a cache line boundary.
            ; .bytes NOP_15
            ; .bytes NOP_15
            ; .bytes NOP_15
            ; .bytes NOP_4

            // Stop any speculative paths from entering our test code
            ; lfence

            // The body of our test is inserted here at compile-time
            ; ->test_body:
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
        .set(2, Event::ExRetBrn(0x00))
        .set(3, Event::BpDeReDirect(0x00));

    //let events = PerfCtlDescriptor::new()
    //    .set(0, Event::LsNotHaltedCyc(0x00))
    //    .set(1, Event::DeDisOpsFromDecoder(0x08));
    //{
    //    // Gadget floor
    //    let code = emit_branch_test_aligned!(scratch_ptr
    //        ; ->branch_target:
    //        ; ->end:
    //    );
    //    run_test(&mut ctx, &events, &code, 
    //             "gadget floor", "/tmp/gadget_floor.txt")?;

    //    // Only JMP
    //    let code = emit_branch_test_aligned!(scratch_ptr
    //        ; jmp ->branch_target
    //        ; ->branch_target:
    //        ; ->end:
    //    );
    //    run_test(&mut ctx, &events, &code, 
    //             "jmp direct (bare)", "/tmp/jmp_bare.txt")?;
    //}


    // An unconditional direct branch (CVE-2021-26341).
    //
    // No prediction occurs when an unconditional direct branch is discovered.
    // Eventually, the branch is completed and the front-end is redirected.

    let code = emit_branch_test_aligned!(scratch_ptr
        ; jmp ->branch_target

        ; prefetch [rdi]
        ; lfence

        ; .align 64
        ; ->branch_target:
        ; ->end:
    );
    run_test(&mut ctx, &events, &code, 
             "jmp direct", "/tmp/jmp_direct.txt")?;

    // It seems like I can fit no more than 9 NOPs here while still causing
    // the prefetch event. 15-byte NOPs don't seem to make a difference.

    let code = emit_branch_test_aligned!(scratch_ptr
        ; jmp ->branch_target

        // It seems like we can only fit 9 NOP instructions?
        //; nop ; nop ; nop ; nop 
        //; nop ; nop ; nop ; nop 
        //; nop

        // We can fit 9 15-byte NOP instructions too??
        ; .bytes NOP_15 ; .bytes NOP_15 ; .bytes NOP_15 ; .bytes NOP_15
        ; .bytes NOP_15 ; .bytes NOP_15 ; .bytes NOP_15 ; .bytes NOP_15
        ; .bytes NOP_15

        // It seems like we could do 8 simple ALU instructions? 
        //; add rax, 1
        //; add rax, 2
        //; add rax, 3
        //; add rax, 4
        //; add rax, 5
        //; add rax, 6
        //; add rax, 7
        //; add rax, 8

        ; prefetch [rdi]
        ; lfence

        ; .align 64
        ; ->branch_target:
        ; ->end:
    );
    run_test(&mut ctx, &events, &code, 
             "jmp direct (9 NOPs) ", "/tmp/jmp_direct_2.txt")?;

    Ok(())
}

