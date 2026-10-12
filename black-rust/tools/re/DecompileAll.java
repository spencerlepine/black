// Ghidra headless post-script: decompile every function (optionally in [lo, hi)) to one .c
// file each, plus index.jsonl (name, addr, size, callees, callers, ok).
// args: <outdir> [lo_hex hi_hex]
// @category black-rust
import ghidra.app.decompiler.DecompInterface;
import ghidra.app.decompiler.DecompileResults;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.Address;
import ghidra.program.model.listing.Function;

import java.io.File;
import java.io.FileWriter;
import java.io.PrintWriter;

public class DecompileAll extends GhidraScript {
    @Override
    public void run() throws Exception {
        String[] args = getScriptArgs();
        File out = new File(args[0]);
        out.mkdirs();
        long lo = args.length > 2 ? Long.parseLong(args[1], 16) : 0;
        long hi = args.length > 2 ? Long.parseLong(args[2], 16) : Long.MAX_VALUE;
        DecompInterface di = new DecompInterface();
        di.openProgram(currentProgram);
        try (PrintWriter idx = new PrintWriter(new FileWriter(new File(out, "index.jsonl")))) {
            for (Function f : currentProgram.getFunctionManager().getFunctions(true)) {
                long a = f.getEntryPoint().getOffset();
                if (a < lo || a >= hi || monitor.isCancelled()) continue;
                DecompileResults r = di.decompileFunction(f, 60, monitor);
                boolean ok = r.decompileCompleted();
                String name = f.getName();
                try (PrintWriter w = new PrintWriter(new FileWriter(new File(out, String.format("%08x_%s.c", a, name))))) {
                    w.println(ok ? r.getDecompiledFunction().getC() : "// decompile failed: " + r.getErrorMessage());
                }
                StringBuilder callees = new StringBuilder();
                for (Function c : f.getCalledFunctions(monitor)) {
                    if (callees.length() > 0) callees.append(',');
                    callees.append('"').append(c.getName()).append('"');
                }
                idx.printf("{\"addr\":\"%08x\",\"name\":\"%s\",\"size\":%d,\"ok\":%b,\"callers\":%d,\"callees\":[%s]}%n",
                    a, name, f.getBody().getNumAddresses(), ok, f.getCallingFunctions(monitor).size(), callees);
            }
        }
        di.dispose();
    }
}
