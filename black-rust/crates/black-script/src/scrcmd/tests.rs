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

/// Ids are unique, handlers are the ROM's table words (bit 0 clear here), and [`handler`] finds each.
#[test]
fn table_is_consistent() {
    let c = commands();
    assert_eq!(c.len(), 49 + 86 + 33 + 29 + 3);
    for w in c.windows(2) {
        assert!(w[0].0 < w[1].0, "duplicate id {}", w[0].0);
    }
    for &(id, a) in &c {
        assert_eq!(handler(id), Some(a));
        assert_eq!(a & 1, 0);
    }
    assert_eq!(handler(100), None);
}

/// Every scrcmd_core / scrcmd_work / scrcmd_talk / scrcmd_screen command id runs through [`dispatch`] (null callees, zeroed memory); the script_sys
/// ones and unknown addresses give `None`.
#[test]
fn every_ported_command_dispatches() {
    let vm = 0x1000;
    let ctx = 0x2000;
    for &(id, a) in &commands() {
        let mut h = Null(Mem::new(0, vec![0; 0x4000]));
        h.0.add_window(crate::scrcmd_screen::WAIT_OBJ_ID, vec![0; 4]);
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
    for slot in [1, 2, 3, 4, 5, 6, 9] {
        let f = crate::script_sys::INIT_PROCS[slot];
        assert_eq!(
            call_init_proc(&mut h, f, 0x3000, 0x300C),
            Some(1),
            "slot {slot}"
        );
    }
    assert_eq!(
        call_init_proc(&mut h, crate::script_sys::INIT_PROCS[0], 0x3000, 0x300C),
        None
    );
    assert!(call_task(
        &mut h,
        crate::scrcmd_screen::FIELD_FADE_TASK,
        0x3000,
        0x3100
    ));
    assert!(!call_task(&mut h, 0x021B_2F18, 0x3000, 0x3100));
}
