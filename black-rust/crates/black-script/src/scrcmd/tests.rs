use super::*;
use crate::hooks::{Mem, MemEnv};
use crate::scrcmd_core::CoreHooks;

/// Every callee answers 0 / does nothing; memory is one zeroed window at 0 (so null pointers stay inside it).
struct Null(Mem);

impl MemEnv for Null {
    fn mem(&mut self) -> &mut Mem {
        &mut self.0
    }
}

#[allow(clippy::too_many_arguments)]
impl CoreHooks for Null {
    fn fun_02011290(&mut self, _vm: u32) {}
    fn fun_02011330(&mut self, _vm: u32) -> u32 {
        0
    }
    fn fun_0201134c(&mut self, _vm: u32) -> u32 {
        0
    }
    fn fun_0201137c(&mut self, _vm: u32, _v: u32) {}
    fn fun_02011398(&mut self, _vm: u32) -> u32 {
        0
    }
    fn fun_020113b0(&mut self, _vm: u32, _pc: u32) {}
    fn fun_020113b4(&mut self, _vm: u32, _pc: u32) {}
    fn fun_020113c4(&mut self, _vm: u32) {}
    fn fun_020113d0(&mut self, _vm: u32, _f: u32) {}
    fn fun_0215a5d4(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5d8(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5dc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5e4(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5f4(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_021587e0(&mut self, _sw: u32, _slot: u32) -> u32 {
        0
    }
    fn fun_0215897c(&mut self, _sw: u32, _child: u32) {}
    fn fun_021589a0(&mut self, _sw: u32, _zone: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02158a34(&mut self, _gamesys: u32, _sw: u32, _id: u32, _mode: u32) {}
    fn fun_02159ae8(&mut self, _vm: u32, _ctx: u32) -> u32 {
        0
    }
    fn fun_02159b10(&mut self, _vm: u32, _ctx: u32) -> u32 {
        0
    }
    fn fun_02159b38(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a684(&mut self, _ctx: u32, _n: u32) {}
    fn fun_0215a690(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215aa24(&mut self, _ctx: u32) {}
    fn fun_0215aa34(&mut self, _ctx: u32) {}
    fn fun_0215aa48(&mut self, _ctx: u32, _slot: u32) {}
    fn fun_0215aa4c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_020120f4(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020122c0(&mut self, _gamesys: u32, _parent: u32, _f: u32, _size: u32) -> u32 {
        0
    }
    fn fun_020124e8(&mut self, _event: u32) -> u32 {
        0
    }
    fn fun_0201296c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012a30(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012b38(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020142e8(&mut self, _ew: u32, _flag: u32) -> u32 {
        0
    }
    fn fun_02014314(&mut self, _ew: u32, _flag: u32) {}
    fn fun_0201433c(&mut self, _ew: u32, _flag: u32) {}
    fn fun_02014460(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02014474(&mut self, _x: u32, _v: u32) {}
    fn fun_020288c4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0202972c(&mut self, _gamesys: u32, _a: u32) -> u32 {
        0
    }
    fn fun_021b1bb8(&mut self, _vm: u32, _ctx: u32) -> u32 {
        0
    }
    fn fun_021b1be8(&mut self, _ctx: u32) {}
}

#[allow(clippy::too_many_arguments)]
impl WorkHooks for Null {
    fn fun_02190988(
        &mut self,
        _fm: u32,
        _header: &[u8; 0x20],
        _list: u32,
        _cb: u32,
        _ctx: u32,
        _a: u32,
        _cursor: u32,
        _flag: u32,
    ) -> u32 {
        0
    }
    fn fun_02017430(
        &mut self,
        _s: u32,
        _l: u32,
        _o: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _h: u32,
    ) -> u32 {
        0
    }
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5d0(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215898c(&mut self, _sw: u32, _arg: u32) {}
    fn fun_02158f54(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f6c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0215909c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_021590d4(&mut self, _sw: u32, _buf: u32) {}
    fn fun_02159198(&mut self, _bit: u32) -> u32 {
        0
    }
    fn fun_02159b94(&mut self, _x: u32, _matrix: u32, _zone: u32, _heap: u32) {}
    fn fun_02159cc4(&mut self, _x: u32, _gamesys: u32, _heap: u32) {}
    fn fun_02159f28(&mut self, _gamedata: u32, _a: u32, _b: u32) {}
    fn fun_02159f54(&mut self, _gamedata: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02159f84(&mut self, _v: u32) -> u32 {
        0
    }
    fn fun_0215a4d8(&mut self, _ew: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0215a4ec(&mut self, _ew: u32, _v: u32) {}
    fn fun_0215a500(&mut self, _ew: u32, _v: u32) {}
    fn fun_0215f6e8(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02162708(&mut self, _x: u32, _a: u32, _b: u32, _c: u32, _d: u32) {}
    fn fun_02162748(&mut self, _x: u32, _a: u32, _b: u32, _c: u32, _d: u32, _e: u32) {}
    fn fun_02162a74(&mut self, _x: u32, _a: u32, _b: u32, _c: u32, _d: u32) {}
    fn fun_021690b4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690d0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690e0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690f0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02169100(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02169110(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0216efa8(&mut self, _x: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0216f830(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0216f948(&mut self, _mon: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0216faa0(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021909bc(&mut self, _handle: u32) {}
    fn fun_021909e4(&mut self, _handle: u32) -> u32 {
        0
    }
    fn fun_02190a20(&mut self, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02190a64(&mut self, _list: u32, _text: u32, _param: u32, _heap: u32) {}
    fn fun_02190ac4(&mut self, _list: u32) -> u32 {
        0
    }
    fn fun_02190acc(&mut self, _fm: u32, _list: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02190aec(&mut self, _rows: u32, _a: u32, _b: u32, _scroll: u32) -> u32 {
        0
    }
    fn fun_02190b08(
        &mut self,
        _header: &mut [u8; 0x20],
        _n: u32,
        _x: u32,
        _y: u32,
        _w: u32,
        _h: u32,
    ) {
    }
    fn fun_02191164(&mut self, _window: u32) {}
    fn fun_02191184(&mut self, _window: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_02197da0(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_02197dac(&mut self, _obj: u32) {}
    fn fun_02030734(&mut self, _heap: u32, _size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_0201f250(&mut self, _wordset: u32, _dst: u32, _src: u32) {}
    fn fun_0201f91c(&mut self, _list: u32) -> u32 {
        0
    }
    fn fun_020202ec(&mut self, _menu: u32, _idx: &mut [u8; 2]) {}
    fn fun_02020d0c(&mut self, _menu: u32, _a: u32) -> u32 {
        0
    }
    fn fun_020457b0(&mut self, _len: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045808(&mut self, _strbuf: u32) {}
    fn fun_02045874(&mut self, _strbuf: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0204589c(&mut self, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02045b38(&mut self, _a: u32, _ty: u32, _msg: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045c04(&mut self, _msgdata: u32) {}
    fn fun_02045cac(&mut self, _msgdata: u32, _id: u32, _strbuf: u32) {}
    fn fun_02045ec0(&mut self, _msgdata: u32, _id: u32) -> u32 {
        0
    }
    fn fun_020056ec(&mut self, _v: u32) -> u32 {
        0
    }
    fn fun_020071e4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020071ec(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200723c(&mut self, _x: u32, _a: &mut [u8; 4], _b: &mut [u8; 4]) {}
    fn fun_0200725c(&mut self, _x: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_020072a4(&mut self, _x: u32, _a: u32) {}
    fn fun_020073d8(&mut self, _x: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_020074d8(&mut self, _boxes: u32, _mon: u32) {}
    fn fun_02007984(&mut self, _boxes: u32) -> u32 {
        0
    }
    fn fun_020079a0(&mut self, _boxes: u32) -> u32 {
        0
    }
    fn fun_02007a68(&mut self, _boxes: u32, _v: u32) {}
    fn fun_020083e4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020083ec(&mut self, _x: u32, _v: u32) {}
    fn fun_02008428(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02008500(&mut self, _player: u32, _strbuf: u32) {}
    fn fun_02008510(&mut self, _player: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02008530(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_02008550(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_02008554(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200873c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020089c0(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020089f4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02008a00(&mut self, _x: u32) {}
    fn fun_02008a7c(&mut self, _x: u32, _v: u32, _t0: u32, _t1: u32, _t2: u32) {}
    fn fun_0200c234(&mut self, _x: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0200c248(&mut self, _x: u32, _v: u32) {}
    fn fun_0200c258(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c274(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c278(&mut self, _x: u32, _v: u32) {}
    fn fun_0200c29c(&mut self, _x: u32, _v: u32) {}
    fn fun_0200c2f0(&mut self, _x: u32, _v: u32, _a: u32) {}
    fn fun_0200c308(&mut self, _x: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200ca28(&mut self, _dex: u32) {}
    fn fun_0200ca38(&mut self, _dex: u32) -> u32 {
        0
    }
    fn fun_0200ca40(&mut self, _dex: u32) {}
    fn fun_0200cde0(&mut self, _dex: u32, _mon: u32) {}
    fn fun_0200ced4(&mut self, _dex: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0200cf28(&mut self, _dex: u32) -> u32 {
        0
    }
    fn fun_0200cfa0(&mut self, _dex: u32, _mon: u32) {}
    fn fun_0200d064(&mut self, _dex: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0200d0cc(&mut self, _dex: u32) -> u32 {
        0
    }
    fn fun_0200ecb4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020127a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012844(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201289c(&mut self, _gamedata: u32, _rec: &[u8; 0x1C]) {}
    fn fun_02012900(&mut self, _gamedata: u32, _v: u32) {}
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201293c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012964(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012974(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012984(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012a44(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012aa0(&mut self, _loc: u32) -> u32 {
        0
    }
    fn fun_02012ac8(&mut self, _loc: u32) -> u32 {
        0
    }
    fn fun_02012ae4(&mut self, _loc: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f5c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02013b44(&mut self, _zone: u32) -> u32 {
        0
    }
    fn fun_02013e4c(&mut self, _zone: u32) -> u32 {
        0
    }
    fn fun_02014184(
        &mut self,
        _rec: &mut [u8; 0x1C],
        _zone: u32,
        _dir: u32,
        _x: u32,
        _y: u32,
        _z: u32,
    ) {
    }
    fn fun_020144dc(&mut self, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_020145cc(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020173f0(&mut self, _species: u32, _level: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02017988(&mut self, _mon: u32, _v: u32) {}
    fn fun_02017a20(&mut self, _mon: u32, _a: u32, _b: u32) {}
    fn fun_02017d30(&mut self, _mon: u32) -> u32 {
        0
    }
    fn fun_02017d70(&mut self, _mon: u32, _k: u32) {}
    fn fun_02017e1c(&mut self, _mon: u32, _param: u32, _out: u32) -> u32 {
        0
    }
    fn fun_02017e40(&mut self, _mon: u32, _param: u32, _v: u32) {}
    fn fun_020180f4(&mut self, _mon: u32, _mv: u32) {}
    fn fun_020181f4(&mut self, _mon: u32, _mv: u32, _slot: u32) {}
    fn fun_020185c4(&mut self, _mon: u32) {}
    fn fun_020185f4(&mut self, _mon: u32) -> u32 {
        0
    }
    fn fun_020186c8(
        &mut self,
        _ot: u32,
        _species: u32,
        _form: u32,
        _a: u32,
        _b: u32,
        _c: u32,
    ) -> u32 {
        0
    }
    fn fun_0201a98c(&mut self, _party: u32, _mon: u32) {}
    fn fun_0201aa14(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201aa1c(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201aa50(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201aa88(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201aad4(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab08(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0201ade4(&mut self, _species: u32, _form: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201ae14(&mut self, _pd: u32) {}
    fn fun_0201ae1c(&mut self, _pd: u32, _field: u32) -> u32 {
        0
    }
    fn fun_0201ba34(&mut self, _party: u32) {}
    fn fun_02020fe4(&mut self, _item: u32, _field: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0202142c(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_02025c88(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0202f190(&mut self, _mon: u32, _a: u32, _player: u32, _b: u32, _heap: u32) {}
    fn fun_020362dc(&mut self) -> u32 {
        0
    }
    fn fun_0203f4e4(&mut self, _t: &mut [u8; 12]) {}
    fn fun_02087d28(&mut self, _t: &mut [u8; 8]) {}
}

#[allow(clippy::too_many_arguments)]
impl TalkHooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5ec(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a60c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a614(&mut self, _ctx: u32, _w: u32) {}
    fn fun_0215a61c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a624(&mut self, _ctx: u32, _v: u32) {}
    fn fun_0215a630(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a63c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a654(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a660(&mut self, _ctx: u32, _v: u32) {}
    fn fun_0215a66c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a678(&mut self, _ctx: u32, _v: u32) {}
    fn fun_0215a77c(
        &mut self,
        _ctx: u32,
        _x: u32,
        _y: u32,
        _cursor: u32,
        _flag6: u32,
        _right_align: u32,
        _var: u32,
        _wordset: u32,
        _msgdata: u32,
    ) {
    }
    fn fun_0215a7f0(&mut self, _ctx: u32, _msg: u32, _help: u32, _param: u32, _s1: u32, _s2: u32) {}
    fn fun_0215a864(&mut self, _ctx: u32) {}
    fn fun_0215a954(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a990(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f78(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f7c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f80(&mut self, _sw: u32, _v: u32) {}
    fn fun_02158f84(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f90(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158fe0(&mut self, _sw: u32, _gamedata: u32, _var: u32) -> u32 {
        0
    }
    fn fun_02159144(&mut self, _bit: u32) {}
    fn fun_02159168(&mut self, _bit: u32) {}
    fn fun_02159190(&mut self, _bit: u32) -> u32 {
        0
    }
    fn fun_020056bc(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_02008550(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c274(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020127a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201d7ec(&mut self, _a: u32, _b: u32, _c: u32, _d: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201d83c(&mut self, _x: u32) {}
    fn fun_0201efa4(
        &mut self,
        _wordset: u32,
        _slot: u32,
        _value: u32,
        _digits: u32,
        _a: u32,
        _b: u32,
    ) {
    }
    fn fun_0201f250(&mut self, _wordset: u32, _dst: u32, _src: u32) {}
    fn fun_0202a310(&mut self, _a: u32, _b: u32, _strbuf: u32, _heap: u32) {}
    fn fun_0202ee30(&mut self, _res: u32, _window: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0202f038(&mut self, _obj: u32) {}
    fn fun_020362dc(&mut self) -> u32 {
        0
    }
    fn fun_020457b0(&mut self, _len: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045808(&mut self, _strbuf: u32) {}
    fn fun_02045b38(&mut self, _a: u32, _ty: u32, _msg: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045c04(&mut self, _msgdata: u32) {}
    fn fun_02045cac(&mut self, _msgdata: u32, _msg: u32, _strbuf: u32) {}
    fn fun_02048ad0(&mut self, _camera: u32) {}
    fn fun_02048ae8(&mut self, _camera: u32, _out: &mut [u8; 12]) {}
    fn fun_02048b0c(&mut self, _camera: u32, _out: &mut [u8; 12]) {}
    fn fun_02048b30(&mut self, _camera: u32, _out: &mut [u8; 12]) {}
    fn fun_0206df3c(&mut self, _pos: &[u8; 12], _y: &mut [u8; 4], _x: &mut [u8; 4]) {}
    fn fun_0207b0a4(&mut self, _m: &[u8; 0x30], _out: &mut [u8; 0x30]) {}
    fn fun_0207b51c(&mut self, _v: &[u8; 12], _m: &[u8; 0x30], _out: &mut [u8; 12]) {}
    fn fun_0207bca0(&mut self, _a: &[u8; 12], _b: &[u8; 12], _c: &[u8; 12], _out: &mut [u8; 0x30]) {
    }
    fn fun_0207ca00(&mut self, _a: &[u8; 12], _b: &[u8; 12], _out: &mut [u8; 12]) {}
    fn fun_0207cae8(&mut self, _a: &[u8; 12], _b: &[u8; 12], _out: &mut [u8; 12]) {}
    fn fun_0207cca4(&mut self, _a: &[u8; 12], _out: &mut [u8; 12]) {}
    fn fun_02082b60(&mut self, _a: &[u8; 0x30], _out: &mut [u8; 0x30]) {}
    fn fun_0216d588(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d5f8(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d88c(&mut self, _mmdl: u32, _out: &mut [u8; 12]) {}
    fn fun_0216de94(&mut self, _mmdls: u32, _obj: u32) -> u32 {
        0
    }
    fn fun_02188c54(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c58(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c84(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d48(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d4c(&mut self, _fm: u32, _board: u32) {}
    fn fun_0218ec2c(&mut self, _camera: u32) -> u32 {
        0
    }
    fn fun_021904dc(&mut self, _msgbg: u32, _on: u32) {}
    fn fun_02190708(
        &mut self,
        _msgbg: u32,
        _msgdata: u32,
        _x: u32,
        _y: u32,
        _w: u32,
        _h: u32,
    ) -> u32 {
        0
    }
    fn fun_02190728(&mut self, _board: u32) {}
    fn fun_02190758(&mut self, _board: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_02190788(&mut self, _board: u32) {}
    fn fun_021909bc(&mut self, _menu: u32) {}
    fn fun_02190be4(&mut self, _msgbg: u32, _arg: u32) -> u32 {
        0
    }
    fn fun_02190c78(&mut self, _menu: u32) -> u32 {
        0
    }
    fn fun_02190fa0(&mut self, _msgbg: u32, _msgdata: u32, _style: u32) -> u32 {
        0
    }
    fn fun_02191008(&mut self, _window: u32) {}
    fn fun_02191084(&mut self, _window: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_021910c0(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_02191134(&mut self, _window: u32) {}
    fn fun_02191164(&mut self, _window: u32) {}
    fn fun_02191184(&mut self, _window: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_021911b4(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_0219129c(
        &mut self,
        _msgbg: u32,
        _tail: u32,
        _pos: u32,
        _text: u32,
        _a: u32,
        _b: u32,
    ) -> u32 {
        0
    }
    fn fun_021912d8(&mut self, _window: u32) {}
    fn fun_021912f8(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_0219131c(&mut self, _window: u32) {}
    fn fun_02191348(&mut self, _window: u32, _text: u32) {}
    fn fun_02191388(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_021914cc(&mut self, _window: u32) {}
    fn fun_02191558(
        &mut self,
        _msgbg: u32,
        _a: u32,
        _kind: u32,
        _b: u32,
        _style: u32,
        _c: u32,
        _d: u32,
    ) -> u32 {
        0
    }
    fn fun_021915b4(&mut self, _window: u32) {}
    fn fun_021915ec(&mut self, _window: u32) {}
    fn fun_021916a8(&mut self, _window: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_021916e0(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_02191794(&mut self, _window: u32) {}
    fn fun_02191830(&mut self, _msgbg: u32, _text: u32, _w: &mut u8, _h: &mut u8) {}
    fn fun_021918dc(
        &mut self,
        _msgbg: u32,
        _text: u32,
        _a: u32,
        _x: u32,
        _y: u32,
        _w: u32,
        _h: u32,
    ) {
    }
    fn fun_0219196c(&mut self, _msgbg: u32, _v: u32) {}
    fn fun_02191978(&mut self, _msgbg: u32) {}
    fn fun_021919ac(&mut self, _msgbg: u32) -> u32 {
        0
    }
    fn fun_02191a04(&mut self, _msgbg: u32, _a: u32) -> u32 {
        0
    }
    fn fun_02191a10(&mut self, _msgbg: u32, _a: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02191b14(&mut self, _window: u32) {}
    fn fun_02191be8(&mut self, _window: u32, _text: u32) -> u32 {
        0
    }
    fn fun_02192570(&mut self, _msgbg: u32, _a: u32, _x: u32, _y: u32, _w: u32, _h: u32) -> u32 {
        0
    }
    fn fun_02192628(&mut self, _window: u32) {}
    fn fun_0219269c(&mut self, _window: u32, _x: u32, _y: u32, _text: u32) {}
    fn fun_021926d8(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_0219270c(&mut self, _msgbg: u32, _text: u32, _a: u32) -> u32 {
        0
    }
    fn fun_0219272c(&mut self, _msgbg: u32, _text: u32, _a: u32) -> u32 {
        0
    }
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
}

#[allow(clippy::too_many_arguments)]
impl ScreenHooks for Null {
    fn fun_02158f50(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012984(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02030734(&mut self, _heap: u32, _size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_02030da8(&mut self, _taskman: u32, _f: u32, _work: u32, _prio: u32) {}
    fn fun_02030eac(&mut self, _task: u32) {}
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_0204e720(&mut self, _mode: u32, _start: u32, _end: u32, _wait: u32) {}
    fn fun_0204e804(&mut self) -> u32 {
        0
    }
    fn fun_02043658(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02043534(&mut self, _x: u32) {}
    fn fun_02188c98(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d20(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d30(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188da0(&mut self, _x: u32) {}
    fn fun_02188db0(&mut self, _fm: u32) {}
    fn fun_02189a1c(&mut self, _fm: u32, _on: u32) {}
    fn fun_02189a24(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_0218dc6c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0218ba98(&mut self, _sys: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0218bacc(&mut self, _sys: u32, _o: u32) -> u32 {
        0
    }
    fn fun_0218c54c(&mut self, _o: u32, _v: u32) {}
    fn fun_0218c574(&mut self, _sys: u32, _o: u32, _v: u32) {}
    fn fun_0218c654(&mut self, _sys: u32, _id: u32, _pos: &[u8; 12]) -> u32 {
        0
    }
    fn fun_0218c69c(&mut self, _sys: u32, _o: u32, _a: u32, _b: u32) {}
    fn fun_0218c7d8(&mut self, _sys: u32, _id: u32, _pos: &[u8; 12]) -> u32 {
        0
    }
    fn fun_0218c860(&mut self, _o: u32) {}
    fn fun_0218c88c(&mut self, _o: u32, _a: u32, _mode: u32) {}
    fn fun_0218c89c(&mut self, _o: u32, _v: u32) {}
    fn fun_0218c8d8(&mut self, _o: u32) -> u32 {
        0
    }
    fn fun_0218c990(&mut self, _o: u32, _se: &mut [u8; 4]) -> u32 {
        0
    }
    fn fun_0218c9a4(&mut self, _o: u32) -> u32 {
        0
    }
    fn fun_021bc5a0(&mut self, _a: u32, _b: u32) {}
    fn fun_021be92c(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_021c0aa0(&mut self, _gamesys: u32, _event: u32) -> u32 {
        0
    }
    fn fun_021c0820(&mut self, _gamesys: u32, _event: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021c978c(&mut self, _event: u32, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_021c9828(&mut self, _event: u32, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_021c98f4(&mut self, _event: u32, _gamesys: u32, _fm: u32, _v: u32) -> u32 {
        0
    }
}

#[allow(clippy::too_many_arguments)]
impl ProcHooks for Null {
    fn fun_02158f50(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158fb4(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158fb8(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158fbc(&mut self, _sw: u32) {}
    fn call_proc_callback(&mut self, _cb: u32, _w: u32) {}
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_0216f940(&mut self, _block: u32) {}
    fn fun_020120b4(&mut self, _gamesys: u32, _ovl: u32, _proc_table: u32, _param: u32) {}
    fn fun_020120f0(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012138(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020124ac(&mut self, _gamesys: u32, _kind: u32, _proc_table: u32, _rec: &[u8]) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012b58(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200be48(&mut self, _save: u32) {}
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0202e474(&mut self, _gamedata: u32, _a: u32, _m: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02014808(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _ovl: u32,
        _proc_table: u32,
        _param: u32,
        _cb: u32,
        _cbwork: u32,
    ) -> u32 {
        0
    }
    fn fun_02014840(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _ovl: u32,
        _proc_table: u32,
        _param: u32,
        _cb: u32,
        _cbwork: u32,
    ) -> u32 {
        0
    }
    fn fun_02034ac4(&mut self, _ovl: u32) {}
    fn fun_02034a5c(&mut self, _ovl: u32) {}
    fn fun_02158344(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0215840c(&mut self, _x: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0215841c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02161b90(&mut self, _gamesys: u32, _event: u32, _a: u32, _b: u32, _c: u32) -> u32 {
        0
    }
    fn fun_02165344(&mut self) {}
    fn fun_0216f918(&mut self, _heap: u32) -> u32 {
        0
    }
    fn fun_0216f948(&mut self, _mon: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02175648(&mut self, _gamedata: u32, _out: u32) {}
    fn fun_02175fb4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021f36c0(&mut self, _gamesys: u32, _a: u32, _b: u32) {}
    fn fun_021f36e8(&mut self, _gamesys: u32, _v: u32) {}
    fn fun_021f3724(&mut self, _gamesys: u32, _v: u32) {}
    fn fun_021f3768(&mut self, _gamesys: u32, _vars: &[u8; 32]) {}
    fn fun_021f3800(&mut self, _gamesys: u32) {}
}

#[allow(clippy::too_many_arguments)]
impl SodateyaHooks for Null {
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012374(&mut self, _event: u32, _child: u32) {}
    fn fun_0200bdb0(&mut self, _save: u32) {}
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200ca38(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02030734(&mut self, _heap: u32, _size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_020144dc(&mut self, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_02014910(&mut self, _gamesys: u32, _fm: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_020145cc(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02188d50(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021be8d4(&mut self, _gamesys: u32, _fm: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021be92c(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_021c3204(&mut self, _daycare: u32, _party: u32, _v: u32) {}
    fn fun_021c3270(&mut self, _daycare: u32, _v: u32, _party: u32) {}
    fn fun_021c32f4(&mut self, _daycare: u32, _party: u32) {}
    fn fun_021c335c(&mut self, _daycare: u32) {}
    fn fun_021c3400(&mut self, _daycare: u32) -> u32 {
        0
    }
    fn fun_021c340c(&mut self, _daycare: u32) -> u32 {
        0
    }
    fn fun_021c341c(&mut self, _daycare: u32, _slot: u32) -> u32 {
        0
    }
    fn fun_021c3490(&mut self, _daycare: u32, _slot: u32) -> u32 {
        0
    }
    fn fun_021c34b4(&mut self, _daycare: u32, _slot: u32) -> u32 {
        0
    }
    fn fun_021c34c4(&mut self, _daycare: u32) -> u32 {
        0
    }
    fn fun_021c34f8(&mut self, _daycare: u32, _slot: u32) -> u32 {
        0
    }
}

#[allow(clippy::too_many_arguments)]
impl Misc10Hooks for Null {
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f50(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02008c58(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02008e68(&mut self, _block: u32, _v: u32) {}
    fn fun_02008e78(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200c0e8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c0f0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c224(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200c22c(&mut self, _block: u32, _n: u32) {}
    fn fun_0200c32c(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200c334(&mut self, _block: u32, _v: u32) {}
    fn fun_0200c33c(&mut self, _block: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200c344(&mut self, _block: u32, _i: u32, _v: u32) {}
    fn fun_0200c358(&mut self, _block: u32, _i: u32, _v: u32) {}
    fn fun_0200c384(&mut self, _block: u32, _lo: u32, _hi: u32) {}
    fn fun_0200e394(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200e3a0(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200e3e8(&mut self, _x: u32, _a: u32, _b: u32) {}
    fn fun_0200e4b0(&mut self, _block: u32, _c: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0200e4f8(&mut self, _block: u32, _c: u32, _k: u32) -> u32 {
        0
    }
    fn fun_02027100(&mut self, _x: u32) {}
    fn fun_02027138(&mut self, _n: u32) {}
    fn fun_0203f524(&mut self) -> u64 {
        0
    }
    fn fun_0200ca1c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200cfa0(&mut self, _dex: u32, _mon: u32) {}
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02017988(&mut self, _mon: u32, _form: u32) {}
    fn fun_021670f8(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02168bf0(&mut self, _gamesys: u32, _rec: u32, _a: u32, _b: u32, _var: u32) -> u32 {
        0
    }
    fn fun_02168cf4(&mut self, _gamesys: u32, _rec: u32) -> u32 {
        0
    }
    fn fun_02168e5c(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_0216d568(&mut self, _mmdl: u32, _flags: u32) {}
    fn fun_0216d88c(&mut self, _mmdl: u32, _out: u32) {}
    fn fun_0216d89c(&mut self, _mmdl: u32, _pos: &[u8; 12]) {}
    fn fun_0216de94(&mut self, _sys: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216aed0(&mut self, _ctx: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0216af1c(&mut self, _ctx: u32, _p: u32, _n: u32, _k: &mut [u8; 4]) -> u32 {
        0
    }
    fn fun_0216af74(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _q: u32,
        _k: u32,
        _var: u32,
        _n: u32,
    ) -> u32 {
        0
    }
    fn fun_0216b8b8(&mut self, _save: u32, _var: u32) {}
    fn fun_0216b91c(&mut self, _save: u32, _v: u32, _var: u32) {}
    fn fun_0216bb18(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021d0a48(&mut self, _gamesys: u32, _rec: u32) {}
    fn fun_021d0a74(&mut self, _rec: u32, _v: u32) {}
    fn fun_021d0ab8(&mut self, _rec: u32, _v: u32) {}
    fn fun_021d0ac0(&mut self, _gamesys: u32, _rec: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021d0bd8(&mut self, _gamesys: u32, _rec: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021d0be4(&mut self, _gamesys: u32, _rec: u32, _var: u32) -> u32 {
        0
    }
    fn fun_021d0db0(&mut self, _gamesys: u32, _rec: u32, _a: u32, _b: u32) {}
    fn fun_021d0fe0(&mut self, _gamesys: u32, _rec: u32) -> u32 {
        0
    }
    fn fun_021d0fe8(&mut self, _gamesys: u32, _rec: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021d1028(&mut self, _gamesys: u32) -> u32 {
        0
    }
}

#[allow(clippy::too_many_arguments)]
impl FldHooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f54(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f88(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f8c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02159124(&mut self, _sw: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02159144(&mut self, _bit: u32) {}
    fn fun_02159168(&mut self, _bit: u32) {}
    fn fun_0215a460(&mut self, _id: u32) -> u32 {
        0
    }
    fn fun_0215a4a0(&mut self, _id: u32) -> u32 {
        0
    }
    fn fun_0215a4b4(&mut self, _id: u32) -> u32 {
        0
    }
    fn fun_0215a4c8(&mut self, _id: u32) -> u32 {
        0
    }
    fn fun_0215a6ac(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a6b8(&mut self, _ctx: u32, _v: u32) {}
    fn fun_0216d588(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216e818(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_0216eab0(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: u32,
    ) -> u32 {
        0
    }
    fn fun_021ae430(&mut self, _gamesys: u32, _ev0: u32, _ev1: u32) -> u32 {
        0
    }
    fn fun_021ae454(&mut self, _heap: u32, _fm: u32, _rec: u32, _n: u32) -> u32 {
        0
    }
    fn fun_021be92c(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c5c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cd0(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d50(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u8; 12]) {}
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021bb3e8(
        &mut self,
        _x: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _loc: &mut [u8; 12],
        _e: u32,
    ) {
    }
    fn fun_021bb578(&mut self, _x: u32, _d: u32, _a: u32, _b: u32) {}
    fn fun_021c34f8(&mut self, _daycare: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02184d38(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u8; 12],
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_02184d90(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _loc: &[u8; 12],
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_02184de8(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _loc: &[u8; 12],
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_02184e40(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _p2: &[u8; 12],
        _zone: u32,
        _pos: &[u8; 12],
    ) -> u32 {
        0
    }
    fn fun_02185050(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u8; 12],
        _d: u32,
    ) -> u32 {
        0
    }
    fn fun_021850a8(&mut self, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_021854b0(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u8; 12],
        _d: u32,
    ) -> u32 {
        0
    }
    fn fun_02185534(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u8; 12],
        _d: u32,
    ) -> u32 {
        0
    }
    fn fun_0216282c(&mut self, _x: u32, _ew: u32, _pos: &[u8; 12], _n: u32) -> u32 {
        0
    }
    fn fun_02162a6c(&mut self, _entry: u32, _p2: &mut [u8; 12]) {}
    fn fun_02012104(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012134(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012374(&mut self, _event: u32, _child: u32) {}
    fn fun_02012844(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012984(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f2c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02008df0(&mut self, _x: u32, _v: u32) {}
    fn fun_02014024(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02013e4c(&mut self, _v: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200c0f0(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c1c4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c1dc(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02028a58(&mut self, _x: u32, _a: u32, _b: u32) {}
    fn fun_020289b8(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02028ba0(&mut self, _x: u32, _v: u32) {}
    fn fun_02028bdc(&mut self, _x: u32, _v: u32) {}
    fn fun_020295a4(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02029620(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02029830(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_020298a0(&mut self, _gamesys: u32, _x: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02029984(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02029a04(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02029a74(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02029af4(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02029df4(&mut self, _gamedata: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02029e20(&mut self, _v: u32) -> u32 {
        0
    }
    fn fun_0202a230(&mut self, _v: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0202b578(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0202c0d0(&mut self, _x: u32, _v: u32) {}
    fn fun_0202c0e4(&mut self, _x: u32, _v: u32) {}
    fn fun_0202c0f8(&mut self, _x: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02005c64(&mut self) -> u32 {
        0
    }
    fn fun_02005f38(&mut self) -> u32 {
        0
    }
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_020061f8(&mut self, _i: u32) {}
    fn fun_02006148(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02006224(&mut self, _i: u32) -> u32 {
        0
    }
    fn fun_02006984(
        &mut self,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: u32,
        _g: u32,
        _tmp: Option<&[u8; 4]>,
    ) -> u32 {
        0
    }
    fn fun_02006bd0(&mut self, _handle: u32) -> u32 {
        0
    }
    fn fun_02006cac(&mut self, _tmp: &mut [u8; 4]) {}
    fn fun_02007e3c(&mut self, _x: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007e50(&mut self, _x: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007f1c(&mut self, _x: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007f68(&mut self, _x: u32, _a: u32, _b: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007fb8(&mut self, _x: u32, _a: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02020fe4(&mut self, _item: u32, _field: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02021288(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020212b0(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201ee84(&mut self, _ws: u32, _idx: u32, _mon: u32) {}
    fn fun_0201eef4(&mut self, _ws: u32, _idx: u32, _mon: u32) {}
    fn fun_0201ef34(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201ef40(&mut self, _ws: u32, _idx: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0201ef4c(&mut self, _ws: u32, _idx: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0201ef5c(&mut self, _ws: u32, _idx: u32, _v: u32) -> u32 {
        0
    }
    fn fun_0201efa4(&mut self, _ws: u32, _idx: u32, _v: u32, _n: u32, _a: u32, _b: u32) {}
    fn fun_0201efc8(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f020(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f02c(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f048(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f14c(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f158(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f184(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f198(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn fun_0201f1b0(&mut self, _ws: u32, _idx: u32, _v: u32) {}
    fn call_wordset_fn(&mut self, _f: u32, _ws: u32, _idx: u32, _v: u32) {}
}

#[allow(clippy::too_many_arguments)]
impl ObjHooks for Null {
    fn fun_0215a5ec(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a6dc(&mut self, _ctx: u32, _m: u32) {}
    fn fun_0215a6fc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f90(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0216ce6c(
        &mut self,
        _mmdls: u32,
        _x: u32,
        _z: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: u32,
        _g: u32,
    ) -> u32 {
        0
    }
    fn fun_0216cee4(
        &mut self,
        _mmdls: u32,
        _a: u32,
        _zone: u32,
        _data: u32,
        _ew: u32,
        _id: u32,
    ) -> u32 {
        0
    }
    fn fun_0216cf2c(&mut self, _mmdl: u32) {}
    fn fun_0216d454(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d588(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d5a0(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d5b0(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d5e0(&mut self, _mmdl: u32, _d: u32) {}
    fn fun_0216d5f8(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d63c(&mut self, _mmdl: u32, _n: u32) -> u32 {
        0
    }
    fn fun_0216d674(&mut self, _mmdl: u32, _v: u32) {}
    fn fun_0216d83c(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d86c(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216d88c(&mut self, _mmdl: u32, _pos: &mut [u8; 12]) {}
    fn fun_0216d89c(&mut self, _mmdl: u32, _pos: &[u8; 12]) {}
    fn fun_0216d8c0(&mut self, _mmdl: u32, _v: &[u8; 12]) {}
    fn fun_0216d9a0(&mut self, _mmdls: u32) {}
    fn fun_0216d9e4(&mut self, _mmdls: u32) -> u32 {
        0
    }
    fn fun_0216da5c(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0216daf8(&mut self, _mmdl: u32) {}
    fn fun_0216db04(&mut self, _mmdl: u32) {}
    fn fun_0216db80(&mut self, _mmdl: u32, _v: u32) {}
    fn fun_0216dddc(&mut self, _x: u32, _gx: u32, _gz: u32, _y: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_0216de54(&mut self, _mmdls: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216de94(&mut self, _mmdls: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216dee0(&mut self, _mmdls: u32) -> u32 {
        0
    }
    fn fun_0216e084(&mut self, _mmdl: u32, _x: u32, _y: u32, _z: u32, _d: u32) {}
    fn fun_0216e0e8(&mut self, _mmdl: u32, _v: u32) {}
    fn fun_0216e3d0(&mut self, _ew: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021618a0(&mut self, _gamesys: u32, _fm: u32, _a: u32, _rec: &[u8; 8]) -> u32 {
        0
    }
    fn fun_02161ac0(&mut self, _gamesys: u32, _mmdl: u32, _from: &[u8; 12], _to: &[u8; 12]) -> u32 {
        0
    }
    fn fun_02162740(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02162744(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021627c8(&mut self, _x: u32, _ew: u32, _pos: &[u8; 12], _dir: u32) -> u32 {
        0
    }
    fn fun_02163e58(&mut self, _mmdl: u32, _pos: &[u8; 12], _h: &mut [u8; 4]) {}
    fn fun_021642a4(&mut self, _d: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020127a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012844(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ac8(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d38(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02197d3c(&mut self, _mmdl: u32, _list: u32) -> u32 {
        0
    }
    fn fun_0219bba4(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_0219e2c0(&mut self, _mmdl: u32, _t: &[u8; 8]) {}
    fn fun_0219e348(&mut self, _mmdl: u32) -> u32 {
        0
    }
    fn fun_021a2f48(&mut self, _player: u32, _v: u32) {}
    fn fun_021a2f54(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u8; 12]) {}
    fn fun_021a3050(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3098(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3238(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a32a8(&mut self, _player: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021b7dc0(&mut self, _x: u32, _t: &mut [u8; 8]) {}
    fn fun_021b83fc(&mut self, _x: u32, _v1: u32, _v2: u32, _v3: u32) {}
    fn fun_021b8484(&mut self, _x: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_021c474c(&mut self, _gamesys: u32, _fm: u32, _mmdl: u32) -> u32 {
        0
    }
    fn fun_021c6de0(&mut self, _tasks: u32, _task: u32, _a: u32) {}
    fn fun_021c7220(&mut self, _fm: u32, _a: u32, _b: u32, _mmdl: u32) -> u32 {
        0
    }
    fn fun_021c7684(&mut self, _fm: u32, _mmdl: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021c7878(&mut self, _fm: u32, _a: u32, _v: &[u8; 12], _mmdl: u32) -> u32 {
        0
    }
    fn fun_021c78dc(&mut self, _fm: u32, _a: u32, _v: &[u8; 12], _mmdl: u32) -> u32 {
        0
    }
    fn fun_021f3770(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f37a4(&mut self, _fm: u32) {}
    fn fun_021f37c8(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021f37ec(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3834(&mut self, _gamesys: u32, _v: u32, _flag: u32) -> u32 {
        0
    }
    fn fun_021f384c(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3888(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f38c4(&mut self, _gamesys: u32, _v: u32) {}
    fn fun_021f3970(&mut self, _gamesys: u32) {}
    fn fun_021f398c(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3a28(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3a88(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021f3ad0(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021f3b18(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3b5c(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f3d74(&mut self, _gamesys: u32) {}
    fn fun_021f3e24(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f4134(&mut self, _gamesys: u32, _v: u32) {}
    fn fun_021f4184(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f41b0(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f41dc(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021f42a8(&mut self, _gamesys: u32) {}
}

#[allow(clippy::too_many_arguments)]
impl ShopHooks for Null {
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02158fb4(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02188c54(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c58(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021904ac(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020127c4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020071cc(&mut self, _save: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02012f2c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021655d4(&mut self) {}
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_021a3050(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_0218ec54(&mut self, _e: u32) {}
    fn fun_0218ec60(&mut self, _e: u32) {}
    fn fun_0218ecb4(&mut self, _e: u32, _t: &mut [u8; 12]) {}
    fn fun_0218ece4(&mut self, _e: u32, _a: &mut [u8; 12]) {}
    fn fun_0218f05c(&mut self, _e: u32) {}
    fn fun_0218f0a8(&mut self, _e: u32) {}
    fn fun_0218f0e8(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218f1ac(&mut self, _e: u32, _v: u32) {}
    fn fun_0218f1b4(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218f848(&mut self, _e: u32) {}
    fn fun_0218f878(&mut self, _e: u32) {}
    fn fun_0218f918(&mut self, _e: u32, _rec: &[u8; 0x48], _frames: u32) {}
    fn fun_0218f9c4(&mut self, _e: u32, _rec: &[u8; 0x18], _frames: u32) {}
    fn fun_0218fe50(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_02193060(&mut self, _on: u32) -> u32 {
        0
    }
    fn fun_021b35a4(&mut self, _gamesys: u32, _w2: u32, _kind: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021b3b14(&mut self, _w2: u32, _kind: u32) -> u32 {
        0
    }
    fn fun_021b331c(&mut self, _w2: u32, _bp: u32) {}
    fn fun_021b3bd8(&mut self, _w2: u32) {}
    fn fun_021b3fa0(&mut self, _w2: u32) {}
    fn fun_021b3f00(&mut self, _w2: u32) {}
    fn fun_021b40d4(&mut self, _w2: u32) {}
    fn fun_021b43ec(&mut self, _w2: u32) -> u32 {
        0
    }
    fn fun_021b385c(&mut self, _w2: u32, _row: u32, _flag: u32, _item: u32, _msg: u32) {}
    fn fun_021b4714(&mut self, _w2: u32, _msg: u32, _item: u32, _x: u32) {}
    fn fun_021b4804(&mut self, _w2: u32, _flag: u32) {}
    fn fun_0200c258(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200c274(&mut self, _money: u32) -> u32 {
        0
    }
    fn fun_0200dabc(&mut self, _bp: u32) -> u32 {
        0
    }
    fn fun_0200c29c(&mut self, _money: u32, _n: u32) {}
    fn fun_0200da98(&mut self, _bp: u32, _n: u32) {}
    fn fun_02008dac(&mut self, _records: u32, _id: u32, _n: u32) {}
    fn fun_02008df0(&mut self, _records: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02007fb8(&mut self, _bag: u32, _item: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007e3c(&mut self, _bag: u32, _item: u32, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200823c(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_02021288(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_02020fe4(&mut self, _item: u32, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201102c(&mut self, _data: u32) -> u32 {
        0
    }
    fn fun_02045b38(&mut self, _a: u32, _b: u32, _arc: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045c04(&mut self, _msg: u32) {}
    fn fun_0201f81c(&mut self, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201f878(&mut self, _row: u32, _a: u32, _b: u32, _c: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02186670(&mut self, _x: u32, _list: u32, _count: u32, _n: u32, _heap: u32) {}
}
#[allow(clippy::too_many_arguments)]
impl ProcFldHooks for Null {
    fn fun_0215898c(&mut self, _sw: u32, _ev: u32) {}
    fn fun_02158f54(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012138(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021896f4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_020124ac(&mut self, _gamesys: u32, _kind: u32, _proc_table: u32, _rec: &[u8; 8]) -> u32 {
        0
    }
    fn fun_020124ac_at(&mut self, _gamesys: u32, _kind: u32, _proc_table: u32, _param: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201283c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02009ce4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02009a34(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201293c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200873c(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200ca38(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200bac8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200baf4(&mut self, _block: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200bbfc(&mut self, _block: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200bb58(&mut self, _block: u32, _data: u32) {}
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_02014840(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _ovl: u32,
        _proc_table: u32,
        _param: u32,
        _cb: u32,
        _cbwork: u32,
    ) -> u32 {
        0
    }
    fn fun_020146f8(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _ovl: u32,
        _proc_table: u32,
        _param: u32,
    ) -> u32 {
        0
    }
    fn fun_021cd2a4(&mut self, _gamesys: u32, _fm: u32, _a: u32, _mode: u32, _var: u32) -> u32 {
        0
    }
    fn fun_0215f8a8(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02009210(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_020090d8(&mut self, _x: u32, _on: u32) {}
    fn fun_02168bbc(&mut self, _gamesys: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021c5bd0(&mut self, _gamesys: u32, _flag: u32, _var: u32) -> u32 {
        0
    }
    fn fun_021d30b8(&mut self, _gamesys: u32, _var: u32) -> u32 {
        0
    }
    fn fun_02188ce8(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021aa0a0(&mut self, _x: u32, _a: u32, _b: u32, _c: u32) -> u32 {
        0
    }
    fn fun_021be92c(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_02012a44(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0216efcc(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216e95c(&mut self, _gamesys: u32, _fm: u32, _n: u32) -> u32 {
        0
    }
    fn fun_021aab48(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200d418(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200d488(&mut self, _x: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0215e440(&mut self, _gamedata: u32, _i: u32) {}
}
#[allow(clippy::too_many_arguments)]
impl EncountHooks for Null {
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020124e4(&mut self, _event: u32) -> u32 {
        0
    }
    fn fun_02012984(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020490f4(&mut self, _arc: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02049238(&mut self, _handle: u32) -> u32 {
        0
    }
    fn fun_02048f4c(&mut self, _t: &mut [u8; 0x2C], _arc: u32, _member: u32) {}
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_02005728(&mut self, _n: u32) -> u32 {
        0
    }
    fn fun_0215e2b8(&mut self, _rec: &mut [u8; 0xE8], _arc: u32, _id: u32, _zone: u32) -> u32 {
        0
    }
    fn fun_02159144(&mut self, _x: u32) {}
    fn fun_02159168(&mut self, _x: u32) {}
    fn fun_02188c58(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c5c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cfc(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d24(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021bb260(&mut self, _x: u32, _on: u32) {}
    fn fun_021bb990(&mut self, _proc_id: u32, _x: u32, _table: u32) -> u32 {
        0
    }
    fn fun_0218e17c(&mut self, _e: u32, _x: u32) {}
    fn fun_0218ec54(&mut self, _e: u32) {}
    fn fun_0218ec60(&mut self, _e: u32) {}
    fn fun_0218f05c(&mut self, _e: u32) {}
    fn fun_0218f0a8(&mut self, _e: u32) {}
    fn fun_0218f0e8(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218f0f8(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218f818(&mut self, _e: u32) {}
    fn fun_0218f878(&mut self, _e: u32) {}
    fn fun_0218f900(&mut self, _e: u32, _rec: &[u8; 0x48], _n: u32) {}
    fn fun_0218f9c4(&mut self, _e: u32, _rec: &[u8; 0x18], _n: u32) {}
    fn fun_0218fa40(&mut self, _e: u32, _n: u32) {}
    fn fun_0218fe50(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_021d2d50(&mut self, _gamesys: u32, _rec: &[u8; 0x18]) -> u32 {
        0
    }
    fn fun_021f36c0(&mut self, _gamesys: u32, _fm: u32, _n: u32) -> u32 {
        0
    }
    fn fun_021f36c8(&mut self, _n: u32) -> u32 {
        0
    }
}
#[allow(clippy::too_many_arguments)]
impl FldBattleHooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0201ee60(&mut self, _list: u32, _i: u32, _v: u32) {}
    fn fun_0201a354(&mut self, _v: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200ced4(&mut self, _dex: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200bac8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200bba8(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200baf4(&mut self, _block: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02019ed4(&mut self, _reg: u32, _party: u32) -> u32 {
        0
    }
    fn fun_0201a090(&mut self, _reg: u32, _party: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021cc720(
        &mut self,
        _fm: u32,
        _party: u32,
        _bx: u32,
        _a: u32,
        _b: u32,
        _heap: u32,
    ) -> u32 {
        0
    }
    fn fun_021cc810(&mut self, _sel: u32) -> u32 {
        0
    }
    fn fun_021cc7b4(&mut self, _sel: u32, _cancel: &mut [u8; 4]) -> u32 {
        0
    }
}
#[allow(clippy::too_many_arguments)]
impl C03Hooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f7c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0215b4ac(&mut self, _ctx: u32, _slot: u32, _mon: &mut [u8; 4]) -> u32 {
        0
    }
    fn fun_0215b500(&mut self, _mon: u32, _mv: u32) -> u32 {
        0
    }
    fn fun_0215a07c(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_0215a0e8(&mut self, _gamesys: u32, _a: u32, _b: u32, _mask: u32, _mv: u32) -> u32 {
        0
    }
    fn fun_0215a2e4(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _c: u32,
        _a: u32,
        _b: u32,
        _z: u32,
        _d: u32,
    ) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02188c58(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c84(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d30(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_0218ed50(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218fe70(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0216cdb8(&mut self, _x: u32, _angle: u32) {}
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_02030da8(&mut self, _tcbsys: u32, _f: u32, _work: u32, _pri: u32) {}
    fn fun_02030eac(&mut self, _tcb: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012fd8(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0201891c(&mut self, _mon: u32, _k: u32) -> u32 {
        0
    }
    fn fun_020188ac(&mut self, _mon: u32, _data: u32) -> u32 {
        0
    }
    fn fun_02018790(&mut self, _mon: u32) -> u32 {
        0
    }
    fn fun_0202130c(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_0200c11c(&mut self, _z: u32, _species: u32, _form: u32, _egg: u32, _gender: u32) {}
    fn fun_0200c2b0(&mut self, _z: u32, _v: u32, _arr: &[u8; 12]) {}
    fn fun_0200c2d0(&mut self, _z: u32, _v: u32, _arr: &mut [u8; 12]) {}
    fn fun_0201ee60(&mut self, _list: u32, _i: u32, _v: u32) {}
    fn fun_0201ee0c(&mut self, _ws: u32, _i: u32, _buf: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200ca38(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200cf08(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200cf54(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200d0f8(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200d298(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200d2b0(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200d2c4(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200e740(&mut self, _z: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200e7ac(&mut self, _z: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200e7bc(&mut self, _z: u32, _i: u32, _mon: u32) {}
    fn fun_02045924(&mut self, _buf: u32, _src: u32) {}
    fn fun_02045924_str(&mut self, _buf: u32, _s: &[u8; 2]) {}
    fn fun_02045a98(&mut self, _r0: u32) -> u32 {
        0
    }
    fn fun_02007fa0(&mut self, _bag: u32, _pocket: u32) -> u32 {
        0
    }
    fn fun_020081dc(&mut self, _bag: u32, _pocket: u32, _i: u32) -> u32 {
        0
    }
    fn fun_020212b0(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_020056ec(&mut self, _r0: u32) -> u32 {
        0
    }
    fn fun_021c5aac(&mut self, _gamesys: u32, _var: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02166c4c(&mut self, _gamesys: u32, _fm: u32, _mon: u32) -> u32 {
        0
    }
    fn fun_021cbfb0(&mut self, _heap: u32, _i: u32) -> u32 {
        0
    }
    fn fun_021cc040(&mut self, _t: u32) {}
    fn fun_021cc060(&mut self, _t: u32) -> u32 {
        0
    }
    fn fun_021cc068(&mut self, _t: u32) -> u32 {
        0
    }
    fn fun_021cc500(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
}
#[allow(clippy::too_many_arguments)]
impl PokemonFldHooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5d0(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a6c4(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a6d0(&mut self, _ctx: u32, _pc: u32) {}
    fn fun_0215a7f0(&mut self, _ctx: u32, _id: u32, _x: u32, _i: u32, _a: u32, _b: u32) {}
    fn fun_02158f54(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f78(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f7c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020127a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012844(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201293c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02008530(&mut self, _status: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0201ab48(&mut self, _party: u32, _i: u32, _mon: u32) {}
    fn fun_0201ab8c(&mut self, _party: u32, _i: u32, _a: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201a98c(&mut self, _party: u32, _mon: u32) {}
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02017e40(&mut self, _mon: u32, _id: u32, _v: u32) {}
    fn fun_02017988(&mut self, _mon: u32, _form: u32) {}
    fn fun_020185c4(&mut self, _mon: u32) {}
    fn fun_020185f0(&mut self, _mon: u32) -> u32 {
        0
    }
    fn fun_0201747c(&mut self, _boxmon: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007698(&mut self, _bx: u32) -> u32 {
        0
    }
    fn fun_020079bc(&mut self, _bx: u32, _b: u32, _s: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02007a18(&mut self, _bx: u32, _b: u32, _s: u32) -> u32 {
        0
    }
    fn fun_020075b0(&mut self, _bx: u32, _b: u32, _s: u32, _boxmon: u32) {}
    fn fun_02008fec(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02008f08(&mut self, _heap: u32) -> u32 {
        0
    }
    fn fun_02009020(&mut self, _mailbox: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0200903c(&mut self, _mailbox: u32, _x: u32, _slot: u32, _mail: u32) {}
    fn fun_02008eac(&mut self, _mail: u32) {}
    fn fun_0202136c(&mut self, _item: u32) -> u32 {
        0
    }
    fn fun_02030734(
        &mut self,
        _heap: u32,
        _size: u32,
        _from_end: u32,
        _file: u32,
        _line: u32,
    ) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _block: u32) {}
    fn fun_02188c84(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d24(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021bb990(&mut self, _proc_id: u32, _x: u32, _table: u32) -> u32 {
        0
    }
    fn fun_021854b0(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u8; 12],
        _x: u32,
    ) -> u32 {
        0
    }
    fn fun_0200a8a0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200a8ac(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_0200a8c8(&mut self, _s: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200a8f8(&mut self, _s: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200a940(&mut self, _s: u32, _month: u32, _day: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021690a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690d0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690e0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02087c88(&mut self, _date: &mut [u8; 8]) {}
    fn fun_020457b0(&mut self, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045808(&mut self, _buf: u32) {}
    fn fun_02045924(&mut self, _buf: u32, _src: u32) {}
    fn fun_0201ee0c(&mut self, _ws: u32, _i: u32, _buf: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_0201efa4(&mut self, _ws: u32, _i: u32, _n: u32, _digits: u32, _a: u32, _b: u32) {}
    fn fun_0201eec4(&mut self, _ws: u32, _i: u32, _x: u32) {}
    fn fun_0201eed4(&mut self, _ws: u32, _i: u32, _mon: u32) {}
    fn fun_0201ef40(&mut self, _ws: u32, _i: u32, _item: u32) {}
    fn fun_0201f02c(&mut self, _ws: u32, _i: u32, _status: u32) {}
    fn fun_0201f184(&mut self, _ws: u32, _i: u32, _x: u32) {}
    fn fun_0204a298(
        &mut self,
        _arc: u32,
        _member: u32,
        _x: u32,
        _heap: u32,
        _size: &mut [u8; 4],
    ) -> u32 {
        0
    }
    fn fun_02007f68(&mut self, _bag: u32, _item: u32, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02007e50(&mut self, _bag: u32, _item: u32, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200c278(&mut self, _x: u32, _v: u32) {}
    fn fun_0200ca1c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0200cde0(&mut self, _dex: u32, _mon: u32) {}
    fn fun_0200be48(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200bf18(&mut self, _block: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02157e9c(&mut self, _rec: u32, _heap: u32, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0216dd1c(&mut self, _sys: u32, _obj: &mut [u8; 4], _index: &mut [u8; 4]) -> u32 {
        0
    }
    fn fun_0216d588(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d598(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_02162740(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_02162744(&mut self, _s: u32) -> u32 {
        0
    }
}
#[allow(clippy::too_many_arguments)]
impl Misc21Hooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a5ec(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158768(&mut self, _gamesys: u32, _script: u32, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_021587b0(&mut self, _event: u32, _script: u32, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02159058(&mut self, _sw: u32, _a: u32, _b: u32, _c: u32, _d: u32) {}
    fn fun_02159104(&mut self, _sw: u32, _w: u32) {}
    fn fun_0215910c(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02159144(&mut self, _flags: u32) {}
    fn fun_02159168(&mut self, _flags: u32) {}
    fn fun_020124e4(&mut self, _event: u32) -> u32 {
        0
    }
    fn fun_02012374(&mut self, _event: u32, _child: u32) {}
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012844(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201293c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f08(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f2c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f38(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f8c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02013c60(&mut self, _zone: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02017e88(&mut self, _boxmon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_020079bc(&mut self, _bx: u32, _b: u32, _s: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02007b44(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200bac8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200bbec(&mut self, _block: u32, _k: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200be48(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c0f0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c234(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200e878(&mut self, _r0: u32) {}
    fn fun_02008530(&mut self, _status: u32) -> u32 {
        0
    }
    fn fun_02008550(&mut self, _status: u32) -> u32 {
        0
    }
    fn fun_02008554(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_0200856c(&mut self, _status: u32) -> u32 {
        0
    }
    fn fun_02008570(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_02008854(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02008990(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02008b08(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02008b94(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02008b98(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02008c58(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02008dac(&mut self, _counters: u32, _id: u32, _n: u32) {}
    fn fun_02008dfc(&mut self, _counters: u32, _id: u32) -> u32 {
        0
    }
    fn fun_020092d8(&mut self, _z: u32) -> u32 {
        0
    }
    fn fun_020092ec(&mut self, _z: u32) -> u32 {
        0
    }
    fn fun_02009390(&mut self, _z: u32, _i: u32) -> u32 {
        0
    }
    fn fun_020093d8(&mut self, _z: u32) -> u32 {
        0
    }
    fn fun_020093f0(&mut self, _z: u32, _a: u32) {}
    fn fun_02009418(&mut self, _z: u32) -> u32 {
        0
    }
    fn fun_02009428(&mut self, _z: u32, _x: u32) {}
    fn fun_02009470(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02009590(&mut self, _s: u32, _status: u32, _v: u32) -> u32 {
        0
    }
    fn fun_020095c8(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_020095fc(&mut self, _s: u32, _i: u32) {}
    fn fun_02009618(&mut self, _s: u32, _i: u32) {}
    fn fun_02009628(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_02009634(&mut self, _s: u32, _i: u32, _field: u32) -> u32 {
        0
    }
    fn fun_02009698(&mut self, _s: u32, _i: u32, _field: u32, _v: u32) {}
    fn fun_0200973c(&mut self, _s: u32, _i: u32) -> u32 {
        0
    }
    fn fun_020249b0(&mut self, _flags: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02025b64(&mut self, _lang: u32) -> u32 {
        0
    }
    fn fun_02025c74(&mut self, _t: u32) -> u32 {
        0
    }
    fn fun_0201147c(
        &mut self,
        _heap: u32,
        _arc: u32,
        _k: u32,
        _x: u32,
        _work: u32,
        _size: u32,
    ) -> u32 {
        0
    }
    fn fun_02011558(&mut self, _anime: u32) {}
    fn fun_02011590(&mut self, _anime: u32, _step: u32) -> u32 {
        0
    }
    fn fun_02011610(&mut self, _anime: u32, _t: &mut [u8; 12]) {}
    fn fun_02011638(&mut self, _anime: u32, _r: &mut [u8; 12]) {}
    fn fun_0207c9d0(&mut self, _a: &[u8; 12], _b: &[u8; 12], _out: &mut [u8; 12]) {}
    fn fun_0201ee0c(&mut self, _ws: u32, _i: u32, _buf: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_0201ee60(&mut self, _ws: u32, _i: u32, _v: u32) {}
    fn fun_0201efa4(&mut self, _ws: u32, _i: u32, _n: u32, _digits: u32, _a: u32, _b: u32) {}
    fn fun_0201f02c(&mut self, _ws: u32, _i: u32, _status: u32) {}
    fn fun_0201f048(&mut self, _ws: u32, _i: u32, _lang: u32) {}
    fn fun_0201f094(&mut self, _ws: u32, _i: u32, _lang: u32, _v: u32) {}
    fn fun_0201f198(&mut self, _ws: u32, _i: u32, _v: u32) -> u32 {
        0
    }
    fn fun_020457b0(&mut self, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045808(&mut self, _buf: u32) {}
    fn fun_02045994(&mut self, _buf: u32, _s: &[u8; 16], _n: u32) {}
    fn fun_02045994_at(&mut self, _buf: u32, _src: u32, _n: u32) {}
    fn fun_020452e8(&mut self, _window: u32) -> u32 {
        0
    }
    fn fun_020056ec(&mut self, _n: u32) -> u32 {
        0
    }
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_02006148(&mut self, _se: u32) -> u32 {
        0
    }
    fn fun_02006224(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_020362dc(&mut self) -> u32 {
        0
    }
    fn fun_0203f4cc(&mut self, _t: &mut [u8; 16]) {}
    fn fun_02188c54(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c58(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c64(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c84(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188ccc(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cfc(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d24(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188d38(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021bb990(&mut self, _proc_id: u32, _x: u32, _table: u32) -> u32 {
        0
    }
    fn fun_021bba1c(&mut self, _x: u32) {}
    fn fun_021bba6c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216d410(&mut self, _obj: u32, _n: u32) {}
    fn fun_0216d454(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d568(&mut self, _obj: u32, _flag: u32) {}
    fn fun_0216d570(&mut self, _obj: u32, _flag: u32) {}
    fn fun_0216d5f8(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d60c(&mut self, _mmdl: u32, _dir: u32) {}
    fn fun_0216d758(&mut self, _obj: u32) {}
    fn fun_0216d88c(&mut self, _obj: u32, _pos: &mut [u8; 12]) {}
    fn fun_0216d89c(&mut self, _mmdl: u32, _pos: &[u8; 12]) {}
    fn fun_0216d9a0(&mut self, _objsys: u32) {}
    fn fun_0216d9e4(&mut self, _objsys: u32) {}
    fn fun_0216de94(&mut self, _objsys: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216e014(&mut self, _obj: u32, _pos: &[u8; 12], _dir: u32) {}
    fn fun_0216e2e8(&mut self, _objsys: u32, _code: u32, _o: &mut [u8; 0x10]) {}
    fn fun_02162740(&mut self, _evdata: u32) -> u32 {
        0
    }
    fn fun_02162744(&mut self, _evdata: u32) -> u32 {
        0
    }
    fn fun_02162890(&mut self, _evdata: u32, _zone: u32, _pos: &[u8; 12]) -> u32 {
        0
    }
    fn fun_021a0ed4(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021a0f30(&mut self, _x: u32, _mode: u32) {}
    fn fun_021a0f94(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021a0fa8(&mut self, _x: u32) {}
    fn fun_021a01d4(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_021a01fc(&mut self, _s: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u8; 12]) {}
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3098(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a30a4(&mut self, _player: u32, _x: u32) {}
    fn fun_021a35fc(&mut self, _player: u32, _x: &mut [u8; 2], _y: &mut [u8; 2], _z: &mut [u8; 2]) {
    }
    fn fun_021a36bc(&mut self, _player: u32, _p: &mut [u8; 8]) {}
    fn fun_021c6de0(&mut self, _t: u32, _mv: u32, _after: u32) {}
    fn fun_021c71d4(&mut self, _fm: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021c7260(&mut self, _fm: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021c7858(&mut self, _fm: u32, _n: u32, _vec: &[u8; 12]) -> u32 {
        0
    }
    fn fun_021cd440(&mut self, _fm: u32, _zone: &mut [u8; 2]) {}
    fn fun_02184f88(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02184fd4(&mut self, _gamesys: u32, _zone: u32) -> u32 {
        0
    }
    fn fun_021867c4(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02186804(&mut self, _t: u32) {}
    fn fun_02186830(&mut self, _t: u32, _k: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0218683c(&mut self, _u: u32) {}
    fn fun_02186844(&mut self, _u: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0218684c(&mut self, _u: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0218ec30(&mut self, _e: u32, _pos: u32) {}
    fn fun_0218ec54(&mut self, _e: u32) {}
    fn fun_0218ec5c(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0218ec90(&mut self, _e: u32, _pos: &[u8; 12]) {}
    fn fun_0218ed04(&mut self, _e: u32, _off: &[u8; 12]) {}
    fn fun_0218f05c(&mut self, _e: u32) {}
    fn fun_0218f0a8(&mut self, _e: u32) {}
    fn fun_0218f0e8(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_0216bf9c(&mut self, _x: u32, _v: u32, _status: u32) -> u32 {
        0
    }
    fn fun_021690d0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_021690e0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02045b38(&mut self, _a: u32, _b: u32, _c: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045180(
        &mut self,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: u32,
        _g: u32,
    ) -> u32 {
        0
    }
    fn fun_02045770(&mut self, _w: u32) -> u32 {
        0
    }
    fn fun_02043b5c(&mut self, _bmp: u32, _color: u32) {}
    fn fun_02045334(&mut self, _w: u32) {}
    fn fun_02045374(&mut self, _w: u32) {}
    fn fun_0201f73c(&mut self, _w: u32, _a: u32, _b: u32, _c: u32) {}
    fn fun_02045cac(&mut self, _msgdata: u32, _msg: u32, _strbuf: u32) {}
    fn fun_021904ac(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0201c7a4(&mut self, _bmp: u32, _a: u32, _b: u32, _strbuf: u32, _color: u32) {}
    fn fun_02041b6c(&mut self, _x: u32) {}
    fn fun_02045c04(&mut self, _msgdata: u32) {}
    fn fun_02041300(&mut self, _a: u32, _b: u32, _c: u32, _d: u32, _e: u32, _f: u32, _g: u32) {}
    fn fun_021d25a4(&mut self, _w: u32) {}
    fn fun_021d25e8(&mut self, _w: u32) {}
    fn fun_021d25f4(&mut self, _w: u32, _a: u32, _b: u32, _c: u32, _d: u32, _e: u32) {}
    fn fun_021d26ac(&mut self, _w: u32) {}
    fn fun_021f37b8(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f3970(
        &mut self,
        _gamesys: u32,
        _pos: &[u8; 12],
        _vec: &[u8; 12],
        _a: u32,
        _b: u32,
    ) -> u32 {
        0
    }
    fn fun_021f3a44(&mut self, _gamesys: u32, _vec: &[u8; 12]) {}
    fn fun_021f3ac0(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f3adc(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f3cfc(&mut self, _gamesys: u32, _pos: &[u8; 12], _a: u32, _b: u32) -> u32 {
        0
    }
}

// gen:scrcmd_misc10b-null-begin
#[allow(clippy::too_many_arguments)]
impl Misc10bHooks for Null {
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0215901c(&mut self, _sw: u32, _gd: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02158344(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02161cdc(&mut self, _e: u32, _gamesys: u32, _a: u32, _b: u32) {}
    fn fun_0216628c(&mut self, _y: u32, _v: u32) -> u32 {
        0
    }
    fn fun_021664cc(&mut self, _heap: u32, _gamesys: u32, _out: &mut u32) -> u32 {
        0
    }
    fn fun_021665d0(&mut self, _gamesys: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021665f0(&mut self, _gamesys: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_02166618(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216662c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216d888(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216d9a0(&mut self, _objsys: u32) {}
    fn fun_0216d9e4(&mut self, _objsys: u32) {}
    fn fun_0216de94(&mut self, _objsys: u32, _id: u32) -> u32 {
        0
    }
    fn fun_020061e4(&mut self, _se: u32) {}
    fn fun_02007e50(&mut self, _zukan: u32, _id: u32, _a: u32, _b: u32) {}
    fn fun_02007f1c(&mut self, _zukan: u32, _id: u32, _a: u32, _b: u32) {}
    fn fun_0200856c(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_0200873c(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_02009470(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_020095c8(&mut self, _block: u32) -> u32 {
        0
    }
    fn fun_0200973c(&mut self, _block: u32, _i: u32) -> u32 {
        0
    }
    fn fun_02009794(&mut self, _block: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200ca10(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200e0b0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200e18c(&mut self, _y: u32, _v: u32) {}
    fn fun_0200e2bc(&mut self, _save: u32, _word: &u32, _species: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0200e6f0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200e6fc(&mut self, _x: u32, _k: u32, _out: &mut [u32; 3]) {}
    fn fun_0200e714(&mut self, _x: u32, _k: u32, _rec: &[u32; 3]) {}
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012138(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012374(&mut self, _event: u32, _child: u32) {}
    fn fun_02012444(&mut self, _event: u32, _id: u32, _data: u32, _work: u32) {}
    fn fun_020124e4(&mut self, _event: u32) -> u32 {
        0
    }
    fn fun_0201292c(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020129a0(&mut self, _gamedata: u32, _weather: u32) {}
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ed0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f38(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012fa4(&mut self, _gamedata: u32, _x: u32) {}
    fn fun_02012fb0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020144dc(&mut self, _gamesys: u32, _x: u32) -> u32 {
        0
    }
    fn fun_020145cc(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0201ee84(&mut self, _ws: u32, _i: u32, _x: u32) {}
    fn fun_02024044(&mut self, _a: u32, _b: u32, _c: u32, _d: u32, _save: u32, _e: u32) -> u32 {
        0
    }
    fn fun_020240fc(&mut self, _rec: u32) {}
    fn fun_02024108(&mut self, _rec: u32, _v: &[u32; 3]) {}
    fn fun_0202411c(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_02024124(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_02024128(&mut self, _rec: u32, _out: &mut u32) {}
    fn fun_02024134(&mut self, _rec: u32, _out: &mut [u32; 3]) {}
    fn fun_02029bcc(&mut self, _gamesys: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02029c00(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02029cc0(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_0202c794(&mut self, _gamesys: u32, _zone: u32) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _p: u32) {}
    fn fun_02034a5c(&mut self, _id: u32) {}
    fn fun_02034ac4(&mut self, _id: u32) {}
    fn fun_0203f1fc(&mut self, _mon: u32, _limit: u32) -> u32 {
        0
    }
    fn fun_021767a0(&mut self, _x: u32, _gd: u32) -> u32 {
        0
    }
    fn fun_021767f0(&mut self, _x: u32, _gd: u32, _pos: &mut [u32; 3]) {}
    fn fun_02176850(&mut self, _comm: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02176870(&mut self, _comm: u32) -> u32 {
        0
    }
    fn fun_0217688c(&mut self, _comm: u32, _x: u32) {}
    fn fun_0217c274(&mut self, _word: &mut u32, _obj: u32) {}
    fn fun_0217c294(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0217c2a0(&mut self, _k: u32, _gd: u32, _word: &u32) -> u32 {
        0
    }
    fn fun_0217c37c(&mut self, _fm: u32, _w: u32, _rec: u32, _a: u32, _b: u32) {}
    fn fun_02184d38(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u32; 3],
        _a: u32,
        _b: u32,
    ) -> u32 {
        0
    }
    fn fun_021854b0(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _zone: u32,
        _pos: &[u32; 3],
        _dir: u32,
    ) -> u32 {
        0
    }
    fn fun_02188c6c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02189a08(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0219c630(&mut self, _objsys: u32) -> u32 {
        0
    }
    fn fun_021a1bdc(&mut self, _weather: u32, _w: u32) {}
    fn fun_021a1be4(&mut self, _weather: u32, _w: u32) {}
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u32; 3]) {}
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021aa304(&mut self, _rec: u32, _gd: u32, _x: u32) {}
    fn fun_021b6100(
        &mut self,
        _gd: u32,
        _mon: u32,
        _rec: u32,
        _status: u32,
        _zukan: u32,
        _a: u32,
        _b: u32,
        _species: u32,
    ) -> u32 {
        0
    }
    fn fun_021b6158(&mut self, _work: u32) -> u32 {
        0
    }
    fn fun_021b95a0(&mut self, _gd: u32, _result: u32, _mon: u32, _species: u32) -> u32 {
        0
    }
    fn fun_021b95c8(&mut self, _work: u32) {}
    fn fun_021be8d4(&mut self, _gamesys: u32, _fm: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_021be92c(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
    ) -> u32 {
        0
    }
    fn fun_021c87a0(&mut self, _w: u32, _i: u32) -> u32 {
        0
    }
    fn fun_021c87ac(&mut self, _w: u32, _i: u32) -> u32 {
        0
    }
    fn fun_021c87b8(&mut self, _w: u32, _i: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021c87c8(&mut self, _w: u32, _i: u32) -> u32 {
        0
    }
    fn fun_021cf498(&mut self, _x: u32, _event: u32, _a: u32) {}
}
// gen:scrcmd_misc10b-null-end

// gen:scrcmd_misc10c-null-begin
#[allow(clippy::too_many_arguments)]
impl Misc10cHooks for Null {
    fn fun_02008530(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200b3a4(&mut self, _ew: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0200c0e8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c0f0(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200c32c(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_0200c34c(&mut self, _s: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200c37c(&mut self, _s: u32) -> u64 {
        0
    }
    fn fun_0200c38c(&mut self, _s: u32, _k: u32, _v: u32) {}
    fn fun_0200d418(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200d468(&mut self, _x: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0200e394(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200e458(&mut self, _t: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0200e47c(&mut self, _t: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012374(&mut self, _event: u32, _child: u32) {}
    fn fun_020124e4(&mut self, _event: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ec4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012f50(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0202f488(&mut self, _mon: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0202f630(&mut self, _mon: u32, _k: u32) {}
    fn fun_02034a5c(&mut self, _id: u32) {}
    fn fun_02034ac4(&mut self, _id: u32) {}
    fn fun_0203f524(&mut self) -> u64 {
        0
    }
    fn fun_020891e0(&mut self, _rest: &mut [u32; 3], _days: &mut [u32; 3], _lo: u32, _hi: u32) {}
    fn fun_02159144(&mut self, _f: u32) {}
    fn fun_02159168(&mut self, _f: u32) {}
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u32; 3]) {}
    fn fun_021d096c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f3800(&mut self, _gamesys: u32) {}
    fn fun_021f385c(&mut self, _fm: u32) {}
    fn fun_021f38bc(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f39b8(
        &mut self,
        _gamesys: u32,
        _mode: u32,
        _from: &[u32; 3],
        _to: &[u32; 3],
        _frames: u32,
        _x: u32,
    ) -> u32 {
        0
    }
    fn fun_021f3aa0(&mut self, _gamesys: u32, _a: u32, _pos: &[u32; 3]) {}
    fn fun_021f3b40(&mut self, _gamesys: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021f3b68(&mut self, _gamesys: u32, _x: u32) -> u32 {
        0
    }
    fn fun_021f5260(&mut self, _a: u32, _b: u32, _c: u32, _gamesys: u32, _ctx: u32) -> u32 {
        0
    }
    fn fun_021f5550(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_021f558c(&mut self, _fm: u32) {}
}
// gen:scrcmd_misc10c-null-end

// gen:scrcmd_bsubway-null-begin
#[allow(clippy::too_many_arguments)]
impl BsubwayHooks for Null {
    fn fun_020056ec(&mut self, _n: u32) -> u32 {
        0
    }
    fn fun_020071cc(&mut self, _save: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02008550(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200b424(&mut self) {}
    fn fun_0200b5cc(&mut self, _save: u32, _k: u32, _out: &mut u32, _x: u32) -> u32 {
        0
    }
    fn fun_0200b6e0(
        &mut self,
        _gd: u32,
        _k: u32,
        _kind: u32,
        _v: u32,
        _x: u32,
        _a: u32,
        _b: u32,
    ) -> u32 {
        0
    }
    fn fun_0200ba38(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200ba58(&mut self) {}
    fn fun_0200ba68(&mut self, _x: u32) {}
    fn fun_0200bac8(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200baf4(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200bba8(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200d85c(&mut self, _p: u32) {}
    fn fun_0200d868(&mut self, _p: u32, _x: u32) {}
    fn fun_0200d87c(&mut self, _p: u32) -> u32 {
        0
    }
    fn fun_0200d884(&mut self, _p: u32, _id: u32, _out: u32) -> u32 {
        0
    }
    fn fun_0200d914(&mut self, _p: u32, _id: u32, _data: &[u8]) {}
    fn fun_0200d9e8(&mut self, _p: u32, _a: u32, _b: u32, _c: u32) -> u32 {
        0
    }
    fn fun_0200da14(&mut self, _p: u32) {}
    fn fun_0200da28(&mut self, _p: u32) -> u32 {
        0
    }
    fn fun_0200da54(&mut self, _p: u32) -> u32 {
        0
    }
    fn fun_0200da80(&mut self, _s: u32, _bp: u32) {}
    fn fun_0200dac4(&mut self, _s: u32, _mode: u32) -> u32 {
        0
    }
    fn fun_0200daec(&mut self, _s: u32, _mode: u32, _v: u32) {}
    fn fun_0200db08(&mut self, _s: u32, _mode: u32, _v: u32) {}
    fn fun_0200db1c(&mut self, _s: u32, _mode: u32) {}
    fn fun_0200db30(&mut self, _s: u32, _mode: u32) {}
    fn fun_0200db44(&mut self, _s: u32, _mode: u32) -> u32 {
        0
    }
    fn fun_0200db60(&mut self, _s: u32, _mode: u32) {}
    fn fun_0200db80(&mut self, _s: u32, _mode: u32) -> u32 {
        0
    }
    fn fun_0200dba0(&mut self, _s: u32, _a: u32, _b: u32) -> u32 {
        0
    }
    fn fun_0200dbf0(&mut self, _s: u32) {}
    fn fun_0200dbfc(&mut self, _s: u32) {}
    fn fun_0200dc08(&mut self, _s: u32) -> u32 {
        0
    }
    fn fun_0200dc0c(&mut self, _s: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0200dc50(&mut self, _s: u32, _k: u32, _records: u32) {}
    fn fun_0200dc94(&mut self, _s: u32, _p: u32) {}
    fn fun_0200de5c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200de64(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200de94(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200dea8(&mut self, _x: u32, _rec: u32, _n: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200df40(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200df4c(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0200df58(&mut self, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0200df94(&mut self, _list: u32) -> u32 {
        0
    }
    fn fun_0200dfb4(&mut self, _rec: u32) -> u32 {
        0
    }
    fn fun_0200ed48(&mut self, _save: u32) -> u32 {
        0
    }
    fn fun_0200ed54(&mut self, _x: u32, _v: u32) {}
    fn fun_0200fb24(&mut self, _r0: u32) -> u32 {
        0
    }
    fn fun_0200fb38(&mut self, _x: u32) {}
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012124(&mut self, _gamesys: u32, _x: u32) {}
    fn fun_02012134(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020124ac(&mut self, _gamesys: u32, _id: u32, _f: u32, _data: &[u32; 4]) -> u32 {
        0
    }
    fn fun_020127a4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_0201289c(&mut self, _gamedata: u32, _rec: &[u8; 0x1C]) {}
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012944(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ebc(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ed8(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ee4(&mut self, _gamedata: u32, _w: u32) {}
    fn fun_020130b4(&mut self, _x: u32) {}
    fn fun_02014184(&mut self, _rec: &mut [u8; 0x1C], _a: u32, _b: u32, _c: u32, _d: u32, _e: u32) {
    }
    fn fun_02014808(
        &mut self,
        _gamesys: u32,
        _fm: u32,
        _id: u32,
        _procs: u32,
        _work: u32,
        _a: u32,
        _b: u32,
    ) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_02019ed4(&mut self, _a: u32, _party: u32) -> u32 {
        0
    }
    fn fun_0201a354(&mut self, _k: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_0201a920(&mut self, _heap: u32) -> u32 {
        0
    }
    fn fun_0201a954(&mut self, _party: u32, _n: u32) {}
    fn fun_0201a98c(&mut self, _party: u32, _mon: u32) {}
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0202b578(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0202c0d0(&mut self, _x: u32, _i: u32) {}
    fn fun_02030734(&mut self, _heap: u32, _size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        0
    }
    fn fun_020307b0(&mut self, _p: u32) {}
    fn fun_02039338(&mut self) -> u32 {
        0
    }
    fn fun_0203cdbc(&mut self) -> u32 {
        0
    }
    fn fun_0203ceac(&mut self, _x: u32) {}
    fn fun_0203d1b4(&mut self) -> u32 {
        0
    }
    fn fun_0203d378(&mut self) -> u32 {
        0
    }
    fn fun_0203d7bc(&mut self, _x: u32) {}
    fn fun_02087bcc(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02158f54(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158fe0(&mut self, _sw: u32, _gd: u32, _id: u32) -> u32 {
        0
    }
    fn fun_02167ab0(&mut self, _w: u32) {}
    fn fun_02167b00(&mut self, _w: u32) {}
    fn fun_02167b18(&mut self, _w: u32, _x: u32) {}
    fn fun_02167b24(&mut self, _x: u32) {}
    fn fun_02167b34(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02167bfc(&mut self, _w: u32, _a: u32, _b: u32) {}
    fn fun_02167cb4(&mut self, _w: u32) -> u32 {
        0
    }
    fn fun_02167cf4(&mut self, _w: u32, _x: u32) {}
    fn fun_02167d00(&mut self, _w: u32, _var: u32) -> u32 {
        0
    }
    fn fun_021681c8(
        &mut self,
        _x: u32,
        _party: u32,
        _mode: u32,
        _rec: u32,
        _trainer: u32,
        _n: u32,
    ) -> u32 {
        0
    }
    fn fun_021684e8(
        &mut self,
        _rec: u32,
        _species: u32,
        _a: u32,
        _b: u32,
        _iv: u32,
        _i: u32,
        _x: u32,
        _heap: u32,
    ) {
    }
    fn fun_02168878(
        &mut self,
        _rec: u32,
        _id: u32,
        _n: u32,
        _a: crate::scrcmd_bsubway::Buf,
        _b: crate::scrcmd_bsubway::Buf,
        _c: u32,
        _heap: u32,
    ) -> u32 {
        0
    }
    fn fun_021688bc(&mut self, _rec: u32, _id: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02168aec(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_0216c538(&mut self, _w: u32, _gamesys: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0216c640(&mut self, _w: u32, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_0216c6e8(&mut self, _w: u32, _gamesys: u32, _a: u32, _b: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0216c860(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_0216cb48(&mut self, _gamesys: u32, _k: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216ce6c(
        &mut self,
        _objsys: u32,
        _x: u32,
        _z: u32,
        _dir: u32,
        _id: u32,
        _code: u32,
        _a: u32,
        _zone: u32,
    ) -> u32 {
        0
    }
    fn fun_0216d5b4(&mut self, _obj: u32, _k: u32) {}
    fn fun_0216d5e0(&mut self, _obj: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0216d5f8(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d620(&mut self, _obj: u32, _v: u32, _k: u32) {}
    fn fun_0216d63c(&mut self, _obj: u32, _k: u32) -> u32 {
        0
    }
    fn fun_0216d88c(&mut self, _obj: u32, _pos: &mut [u32; 3]) {}
    fn fun_0216dac8(&mut self, _obj: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0216daf8(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216db4c(&mut self, _obj: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0216de94(&mut self, _objsys: u32, _id: u32) -> u32 {
        0
    }
    fn fun_0216e014(&mut self, _obj: u32, _pos: &[u32; 3], _dir: u32) {}
    fn fun_02188c54(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c9c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021909bc(&mut self, _x: u32) {}
    fn fun_02190be4(&mut self, _x: u32, _v: u32) -> u32 {
        0
    }
    fn fun_02190c78(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021a3038(&mut self, _player: u32, _pos: &mut [u32; 3]) {}
    fn fun_021a3050(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a30b0(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3190(&mut self, _x: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021a31a4(&mut self, _x: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021c9924(&mut self, _gamesys: u32) {}
    fn fun_021cd48c(
        &mut self,
        _status: u32,
        _fm: u32,
        _a: u32,
        _b: u32,
        _c: u32,
        _d: u32,
        _e: u32,
        _f: u32,
    ) -> u32 {
        0
    }
    fn fun_021cd5a4(&mut self, _menu: u32) {}
    fn fun_021cd688(&mut self, _menu: u32) -> u32 {
        0
    }
    fn fun_021d024c(&mut self, _x: u32, _a: u32) -> u32 {
        0
    }
    fn fun_021d028c(&mut self, _x: u32, _a: u32) -> u32 {
        0
    }
    fn fun_021eee78(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021f37c0(&mut self, _fm: u32, _a: u32, _rec: u32) -> u32 {
        0
    }
    fn fun_021f380c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_021f388c(&mut self, _fm: u32) {}
}
// gen:scrcmd_bsubway-null-end

// gen:scrcmd_misc21b-null-begin
#[allow(clippy::too_many_arguments)]
impl Misc21bHooks for Null {
    fn fun_0201210c(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_02012138(&mut self, _gamesys: u32) -> u32 {
        0
    }
    fn fun_020127c4(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_020127d0(&mut self, _gamedata: u32, _k: u32) -> u32 {
        0
    }
    fn fun_020127f0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012934(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02012ed0(&mut self, _gamedata: u32) -> u32 {
        0
    }
    fn fun_02017e1c(&mut self, _mon: u32, _id: u32, _x: u32) -> u32 {
        0
    }
    fn fun_0201aa18(&mut self, _party: u32) -> u32 {
        0
    }
    fn fun_0201ab28(&mut self, _party: u32, _i: u32) -> u32 {
        0
    }
    fn fun_0201ec64(&mut self, _heap: u32) -> u32 {
        0
    }
    fn fun_02030734(&mut self, _heap: u32, _size: u32, _flag: u32, _file: u32, _line: u32) -> u32 {
        0
    }
    fn fun_02034a5c(&mut self, _id: u32) {}
    fn fun_02034ac4(&mut self, _id: u32) {}
    fn fun_0203f4e4(&mut self, _date: &mut [u32; 3]) {}
    fn fun_020457b0(&mut self, _len: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02045b38(&mut self, _a: u32, _b: u32, _c: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_02158344(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_02158434(&mut self, _r0: u32) -> u32 {
        0
    }
    fn fun_02158f58(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_02158f74(&mut self, _sw: u32) -> u32 {
        0
    }
    fn fun_0215a5cc(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_0215a644(&mut self, _ctx: u32, _task: u32) {}
    fn fun_0215a64c(&mut self, _ctx: u32) -> u32 {
        0
    }
    fn fun_021666a4(&mut self, _slot: u32) {}
    fn fun_0216c204(&mut self, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_0216d588(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d5f8(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_0216d88c(&mut self, _obj: u32, _pos: u32) {}
    fn fun_0216ddc0(&mut self, _objsys: u32, _x: u32, _z: u32, _k: u32) -> u32 {
        0
    }
    fn fun_02186090(&mut self, _obj: u32) -> u32 {
        0
    }
    fn fun_02186278(&mut self, _data: u32, _k: u32) -> u32 {
        0
    }
    fn fun_02188c54(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c7c(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c80(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c84(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188c98(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cb4(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02188cd0(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_02189a00(&mut self, _fm: u32) -> u32 {
        0
    }
    fn fun_0218dc70(&mut self, _x: u32) -> u32 {
        0
    }
    fn fun_021938f4(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_02193904(&mut self, _e: u32) {}
    fn fun_0219390c(&mut self, _e: u32, _rec: u32) -> u32 {
        0
    }
    fn fun_0219391c(&mut self, _e: u32) -> u32 {
        0
    }
    fn fun_02193b40(&mut self, _e: u32, _rec: u32) {}
    fn fun_02193b68(&mut self, _e: u32, _ws: u32, _rec: u32, _v: u32, _heap: u32) {}
    fn fun_02193bc4(&mut self, _e: u32, _rec: u32) {}
    fn fun_02193be4(&mut self, _e: u32, _rec: u32) -> u32 {
        0
    }
    fn fun_02193c00(&mut self) -> u32 {
        0
    }
    fn fun_02193c04(&mut self, _person: u32) -> u32 {
        0
    }
    fn fun_021a3050(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3074(&mut self, _player: u32) -> u32 {
        0
    }
    fn fun_021a3364(&mut self, _player: u32, _dir: u32, _pos: &mut [u32; 3]) {}
    fn fun_021a3460(&mut self, _player: u32, _x: &mut [u8; 4], _y: &mut [u8; 2], _z: &mut [u8; 2]) {
    }
    fn fun_021ab0d4(&mut self, _map: u32, _pos: &[u32; 3]) -> u32 {
        0
    }
    fn fun_021c1024(&mut self, _player: u32, _dir: u32, _x: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_021c1094(&mut self, _task: u32) -> u32 {
        0
    }
    fn fun_021c10a0(&mut self, _task: u32) {}
    fn fun_021c1228(&mut self, _player: u32, _dir: u32, _v: u32, _heap: u32) -> u32 {
        0
    }
    fn fun_021c12c4(&mut self, _task: u32) -> u32 {
        0
    }
    fn fun_021c12d0(&mut self, _task: u32) {}
    fn fun_021c1cb0(&mut self, _gamesys: u32, _x: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021c1d68(&mut self, _gamesys: u32, _x: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021c30b8(&mut self, _gamesys: u32, _x: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021c9550(&mut self, _obj: u32, _x: u32) {}
    fn fun_021d2418(&mut self, _gamesys: u32, _fm: u32) -> u32 {
        0
    }
    fn fun_021d2440(&mut self, _gamesys: u32, _fm: u32, _k: u32) -> u32 {
        0
    }
    fn fun_021f36c0(&mut self, _w: u32) {}
}
// gen:scrcmd_misc21b-null-end

/// Ids are unique, handlers are the ROM's table words (bit 0 clear here), and [`handler`] finds each.
#[test]
fn table_is_consistent() {
    let c = commands();
    // F8a..F8d: core 49, work 86, talk 33, screen 29, fld 80, obj 55, proc 17, sodateya 15, misc10 29; F8e: shop 1,
    // proc_fld 20, encount 14, fld_battle 2, c03center 22, pokemon_fld 13 (494 is X2), misc21 36; F8f: misc10b 13,
    // misc10c 17, bsubway 4, misc21b 23; script_sys 3
    assert_eq!(
        c.len(),
        49 + 86
            + 33
            + 29
            + 80
            + 55
            + 17
            + 15
            + 29
            + 1
            + 20
            + 14
            + 2
            + 22
            + 13
            + 36
            + 13
            + 17
            + 4
            + 23
            + 3
    );
    for w in c.windows(2) {
        assert!(w[0].0 < w[1].0, "duplicate id {}", w[0].0);
    }
    for &(id, a) in &c {
        assert_eq!(handler(id), Some(a));
        assert_eq!(a & 1, 0);
    }
    assert_eq!(handler(93), None);
    // command 359 (scrcmd_musical.c, X2) and 494 (Dream World, X2) are specced only
    assert_eq!(handler(359), None);
    assert_eq!(handler(494), None);
    assert_eq!(handler(414), Some(0x021D_29F0));
    assert_eq!(handler(329), Some(0x021B_2FD8));
    assert_eq!(handler(600), Some(0x021D_3090));
    assert_eq!(
        handler(crate::scrcmd_misc10::COMMANDS[0].0),
        Some(0x0216_A420)
    );
}

/// Every scrcmd_core / scrcmd_work / scrcmd_talk / scrcmd_screen / scrcmd_fld / scrcmd_obj / scrcmd_proc / scrcmd_sodateya /
/// scrcmd_misc10 command id runs
/// through [`dispatch`] (null callees, zeroed memory); the script_sys
/// ones and unknown addresses give `None`.
#[test]
fn every_ported_command_dispatches() {
    let vm = 0x1000;
    let ctx = 0x2000;
    for &(id, a) in &commands() {
        let mut h = Null(Mem::new(0, vec![0; 0x4000]));
        h.0.add_window(crate::scrcmd_screen::WAIT_OBJ_ID, vec![0; 4]);
        h.0.add_window(crate::scrcmd_c03center::TURN_ANGLE, vec![0; 4]);
        h.w32(vm + crate::vm::VM_PC, 0x3000);
        h.w32(vm + crate::vm::VM_REGS, 0x3800);
        h.w32(ctx + crate::scrcmd_work::CTX_WORK, 0x2800);
        let r = dispatch(&mut h, a | 1, vm, ctx);
        if SCRIPT_SYS_COMMANDS.iter().any(|c| c.0 == id) {
            assert_eq!(r, None, "command {id}");
        } else {
            assert!(matches!(r, Some(0 | 1)), "command {id}: {r:?}");
        }
    }
    let mut h = Null(Mem::new(0, vec![0; 0x4000]));
    h.0.add_window(crate::scrcmd_screen::WAIT_OBJ_ID, vec![0; 4]);
    assert_eq!(dispatch(&mut h, 0x021B_1BB9, vm, ctx), None);
}

/// The two wait callbacks of the ported commands are routed (the null countdown hook says "not done", the null VM-slot
/// hook says "ended").
#[test]
fn waits_dispatch() {
    let mut h = Null(Mem::new(0, vec![0; 0x4000]));
    h.0.add_window(crate::scrcmd_screen::WAIT_OBJ_ID, vec![0; 4]);
    h.w32(0x2000 + crate::scrcmd_work::CTX_WORK, 0x2800);
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_core::WAIT_COUNTDOWN, 0x1000, 0x2000),
        Some(0)
    );
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_core::WAIT_CALL_SCRIPT, 0x1000, 0x2000),
        Some(1)
    );
    assert_eq!(call_wait(&mut h, 0x0215_9961, 0x1000, 0x2000), None);
    // overlay 21: the null window polls never say "done"; the null fade / object hooks say "not running"
    for f in [
        crate::scrcmd_talk::WAIT_TALK,
        crate::scrcmd_talk::WAIT_BALLOON_CLOSE,
        crate::scrcmd_talk::WAIT_BALLOON,
        crate::scrcmd_talk::WAIT_SYSTEM,
        crate::scrcmd_talk::WAIT_WINDOW_67,
        crate::scrcmd_talk::WAIT_WINDOW_69,
    ] {
        assert_eq!(call_wait(&mut h, f, 0x1000, 0x2000), Some(0), "{f:#x}");
    }
    // no key and no window open: command 75's wait ends
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_talk::WAIT_BUTTON, 0x1000, 0x2000),
        Some(1)
    );
    for f in [
        crate::scrcmd_screen::WAIT_FADE,
        crate::scrcmd_screen::WAIT_FIELD_FADE,
        crate::scrcmd_screen::WAIT_298,
    ] {
        assert_eq!(call_wait(&mut h, f, 0x1000, 0x2000), Some(1), "{f:#x}");
    }
    // the null proc-running hook says "not running": the app-proc wait ends
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_proc::PROC_WAIT, 0x1000, 0x2000),
        Some(1)
    );
    // scrcmd_fld / scrcmd_obj waits (null callees: "still busy" = 0 is "done" for the `== 0` polls)
    for f in [
        crate::scrcmd_fld::WAIT_160,
        crate::scrcmd_fld::WAIT_168,
        crate::scrcmd_fld::WAIT_172,
        crate::scrcmd_obj::WAIT_MOVEMENTS,
    ] {
        assert_eq!(call_wait(&mut h, f, 0x1000, 0x2000), Some(1), "{f:#x}");
    }
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_obj::WAIT_117, 0x1000, 0x2000),
        Some(0)
    );
    // the shop (F8e): state 0 starts the camera and waits
    assert_eq!(
        call_wait(&mut h, crate::scrcmd_shop::WAIT_329, 0x1000, 0x2000),
        Some(0)
    );
}

/// The GMEVENT functions, init procs and the task function of the overlay-21 commands are routed; other addresses are not.
#[test]
fn events_procs_tasks_dispatch() {
    let mut h = Null(Mem::new(0, vec![0; 0x4000]));
    h.0.add_window(crate::scrcmd_screen::WAIT_OBJ_ID, vec![0; 4]);
    h.w32(0x2000, 0x2800);
    for f in [
        crate::scrcmd_talk::EVENT_YES_NO,
        crate::scrcmd_talk::EVENT_MENU,
    ] {
        assert_eq!(
            call_event(&mut h, f, 0x1000, 0x1800, 0x2000),
            Some(0),
            "{f:#x}"
        );
    }
    assert_eq!(
        call_event(&mut h, 0x021B_0131, 0x1000, 0x1800, 0x2000),
        None
    );
    // overlay 10 (F8d): seq 0 of each event is one step that does not finish
    for f in [
        crate::scrcmd_sodateya::EVENT_247,
        crate::scrcmd_misc10::EVENT_550,
    ] {
        h.w32(0x1800, 0);
        assert_eq!(
            call_event(&mut h, f, 0x1000, 0x1800, 0x2000),
            Some(0),
            "{f:#x}"
        );
    }
    // command 549's move: 1 frame -> finished after one step
    h.w16(0x2000 + 4, 1);
    assert_eq!(
        call_event(
            &mut h,
            crate::scrcmd_misc10::EVENT_549,
            0x1000,
            0x1800,
            0x2000
        ),
        Some(1)
    );
    for cb in [
        crate::scrcmd_proc::CB_334,
        crate::scrcmd_proc::CB_336,
        crate::scrcmd_proc::CB_489,
    ] {
        assert!(call_callback(&mut h, cb, 0x3000), "{cb:#x}");
    }
    assert!(!call_callback(
        &mut h,
        crate::scrcmd_proc::CB_334 & !1,
        0x3000
    ));
    for slot in [1, 2, 3, 4, 5, 6, 9] {
        let f = crate::script_sys::INIT_PROCS[slot];
        assert_eq!(
            call_init_proc(&mut h, f, 0x3000, 0x300C),
            Some(1),
            "slot {slot}"
        );
    }
    // F8e: slot 0 (scrcmd_encount) turns the field effect off and ends; slot 8 (scrcmd_misc21) waits for its object
    assert_eq!(
        call_init_proc(&mut h, crate::script_sys::INIT_PROCS[0], 0x3000, 0x300C),
        Some(1)
    );
    h.w32(0x300C, 0);
    assert_eq!(
        call_init_proc(&mut h, crate::script_sys::INIT_PROCS[8], 0x3000, 0x300C),
        Some(0)
    );
    assert_eq!(
        call_init_proc(&mut h, crate::script_sys::INIT_PROCS[7], 0x3000, 0x300C),
        None
    );
    // F8e events: the party selection (seq 0 opens it), the encount waits (null object: not busy -> done), command 350's
    // wait (null animation: not running -> done)
    h.w32(0x1800, 0);
    assert_eq!(
        call_event(
            &mut h,
            crate::scrcmd_fld_battle::EVENT_SELECT,
            0x1000,
            0x1800,
            0x2000
        ),
        Some(0)
    );
    for f in [
        crate::scrcmd_encount::EVENT_319,
        crate::scrcmd_encount::EVENT_325,
        crate::scrcmd_misc21::EVENT_350,
    ] {
        h.w32(0x2000, 0x2400);
        assert_eq!(
            call_event(&mut h, f, 0x1000, 0x1800, 0x2000),
            Some(1),
            "{f:#x}"
        );
    }
    assert_eq!(
        call_event(
            &mut h,
            crate::scrcmd_misc21::EVENT_SHOW_MON,
            0x1000,
            0x1800,
            0x2000
        ),
        None
    );
    assert!(call_callback(
        &mut h,
        crate::scrcmd_proc_fld::CB_335,
        0x3000
    ));
    h.0.add_window(crate::scrcmd_c03center::TURN_ANGLE, vec![0; 4]);
    h.w32(0x3100 + 0xC, 1);
    assert!(call_task(
        &mut h,
        crate::scrcmd_c03center::TURN_TASK,
        0x3000,
        0x3100
    ));
    assert!(call_field_proc(
        &mut h,
        crate::scrcmd_misc21::ANIM_PROC_STEP,
        0,
        0,
        0x3100
    ));
    assert!(!call_field_proc(&mut h, 0x021D_18F4, 0, 0, 0x3100));
    assert!(call_task(
        &mut h,
        crate::scrcmd_screen::FIELD_FADE_TASK,
        0x3000,
        0x3100
    ));
    assert!(!call_task(&mut h, 0x021B_2F18, 0x3000, 0x3100));
}

/// F8f: the wait callbacks of commands 346 (bsubway) and 197 / 198, the GMEVENT functions of scrcmd_misc10b / scrcmd_misc10c /
/// scrcmd_bsubway are routed (null callees: each runs one step without panicking).
#[test]
fn f8f_waits_events_dispatch() {
    let mut h = Null(Mem::new(0, vec![0; 0x4000]));
    for f in [
        crate::scrcmd_bsubway::WAIT_6DC,
        crate::scrcmd_bsubway::WAIT_700,
        crate::scrcmd_bsubway::WAIT_754,
        crate::scrcmd_bsubway::WAIT_7E8,
        crate::scrcmd_bsubway::WAIT_86C,
        crate::scrcmd_bsubway::WAIT_C14,
        crate::scrcmd_misc21b::WAIT_197,
        crate::scrcmd_misc21b::WAIT_198,
    ] {
        assert!(
            matches!(call_wait(&mut h, f, 0x1000, 0x2000), Some(0 | 1)),
            "{f:#x}"
        );
    }
    for f in [
        crate::scrcmd_misc10b::EVENT_BOAT,
        crate::scrcmd_misc10b::EVENT_480,
        crate::scrcmd_misc10b::EVENT_493,
        crate::scrcmd_misc10b::EVENT_SYMBOL_BATTLE,
        crate::scrcmd_misc10b::EVENT_INTRUDE_WAIT,
        crate::scrcmd_misc10b::EVENT_FOREST_WARP,
        crate::scrcmd_misc10c::EVENT_468,
        crate::scrcmd_bsubway::EVENT_DISCONNECT,
        crate::scrcmd_bsubway::EVENT_934,
    ] {
        h.w32(0x1800, 0);
        assert!(
            matches!(call_event(&mut h, f, 0x1000, 0x1800, 0x2000), Some(0 | 1)),
            "{f:#x}"
        );
    }
    assert_eq!(
        call_event(
            &mut h,
            crate::scrcmd_misc21b::EVENT_WFBC_TALK,
            0x1000,
            0x1800,
            0x2000
        ),
        None
    );
}
