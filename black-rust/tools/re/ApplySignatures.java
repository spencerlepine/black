// Ghidra headless script: apply C prototypes, then re-run decompiler parameter ID on callers.
// args: <file>...  a *.h file is parsed first with Ghidra's CParser into the program's data types
// (structs usable in prototypes, e.g. BTL_POKEPARAM *); every other file is a .tsv applied in
// order (later lines win). tsv lines: <hex addr or symbol>\t<C prototype>   ('#' = comment)
// A line "@pool\t<lo_hex>\t<hi_hex>\t<C prototype>" applies the prototype to every Thumb
// function pointer in the (u32 event, u32 fn) pair table [lo, hi).
// @category black-rust
import ghidra.app.cmd.function.ApplyFunctionSignatureCmd;
import ghidra.app.script.GhidraScript;
import ghidra.app.util.cparser.C.CParserUtils;
import ghidra.program.model.address.Address;
import ghidra.program.model.data.DataType;
import ghidra.program.model.data.FunctionDefinitionDataType;
import ghidra.program.model.listing.Function;
import ghidra.program.model.symbol.SourceType;
import ghidra.program.model.symbol.Symbol;

import java.nio.file.Files;
import java.nio.file.Paths;

public class ApplySignatures extends GhidraScript {
    private int applied, failed;

    private void apply(Address a, String proto) throws Exception {
        Function f = getFunctionAt(a);
        if (f == null) f = createFunction(a, null);
        if (f == null) { failed++; return; }
        // keep the decomp's symbol name; only take the types from the prototype
        String named = proto.replaceFirst("\\b\\w+\\s*\\(", f.getName() + "(");
        FunctionDefinitionDataType sig = CParserUtils.parseSignature(null, currentProgram, named, true);
        if (sig == null) { failed++; return; }
        if (new ApplyFunctionSignatureCmd(a, sig, SourceType.USER_DEFINED).applyTo(currentProgram, monitor)) applied++;
        else failed++;
    }

    private Address resolve(String s) {
        if (s.matches("(0x)?[0-9a-fA-F]{8}")) return toAddr(Long.parseLong(s.replace("0x", ""), 16));
        for (Symbol sym : currentProgram.getSymbolTable().getSymbols(s)) return sym.getAddress();
        return null;
    }

    private void loadTypes(String header) throws Exception {
        int before = currentProgram.getDataTypeManager().getDataTypeCount(true);
        CParserUtils.CParseResults r = CParserUtils.parseHeaderFiles(null, new String[] { header },
            new String[0], currentProgram.getDataTypeManager(), monitor);
        if (r == null || !r.successful()) { println("ApplySignatures: types FAILED " + header + ": " + (r == null ? "null" : r.getFormattedParseMessage(null))); return; }
        int after = currentProgram.getDataTypeManager().getDataTypeCount(true);
        println("ApplySignatures: types added " + (after - before) + " from " + header);
        java.util.List<DataType> found = new java.util.ArrayList<>();
        currentProgram.getDataTypeManager().findDataTypes("BTL_POKEPARAM", found);
        for (DataType dt : found) println("ApplySignatures: " + dt.getPathName() + " length 0x" + Integer.toHexString(dt.getLength()));
    }

    @Override
    public void run() throws Exception {
        for (String arg : getScriptArgs()) if (arg.endsWith(".h")) loadTypes(arg);
        for (String arg : getScriptArgs()) if (!arg.endsWith(".h")) applyFile(arg);
        println("ApplySignatures: applied " + applied + ", failed " + failed);
    }

    private void applyFile(String path) throws Exception {
        for (String line : Files.readAllLines(Paths.get(path))) {
            if (line.isBlank() || line.startsWith("#")) continue;
            String[] p = line.split("\t");
            if (p[0].equals("@pool")) {
                long lo = Long.parseLong(p[1], 16), hi = Long.parseLong(p[2], 16);
                for (long x = lo; x < hi; x += 8) {
                    long fn = getInt(toAddr(x + 4)) & 0xFFFFFFFFL;
                    apply(toAddr(fn & ~1L), p[3]);
                }
            } else {
                Address a = resolve(p[0]);
                if (a == null) { failed++; continue; }
                apply(a, p[1]);
            }
        }
    }
}
