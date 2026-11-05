/*!
 * ZBra docs theme — tiny chess core.
 * Just enough rules to replay a SAN game with correct disambiguation.
 * Squares are 0..63, index = rank * 8 + file (a1 = 0, h8 = 63).
 */
(function (root) {
  'use strict';

  // Donald Byrne – Robert J. Fischer, Rosenwald Memorial, New York, 17 Oct 1956.
  var GAME = {
    white: 'Donald Byrne',
    black: 'Robert J. Fischer',
    event: 'Rosenwald Memorial',
    place: 'New York',
    date: '17 October 1956',
    result: '0–1',
    moves: (
      'Nf3 Nf6 c4 g6 Nc3 Bg7 d4 O-O Bf4 d5 Qb3 dxc4 Qxc4 c6 e4 Nbd7 Rd1 Nb6 Qc5 Bg4 ' +
      'Bg5 Na4 Qa3 Nxc3 bxc3 Nxe4 Bxe7 Qb6 Bc4 Nxc3 Bc5 Rfe8+ Kf1 Be6 Bxb6 Bxc4+ Kg1 Ne2+ ' +
      'Kf1 Nxd4+ Kg1 Ne2+ Kf1 Nc3+ Kg1 axb6 Qb4 Ra4 Qxb6 Nxd1 h3 Rxa2 Kh2 Nxf2 Re1 Rxe1 ' +
      'Qd8+ Bf8 Nxe1 Bd5 Nf3 Ne4 Qb8 b5 h4 h5 Ne5 Kg7 Kg1 Bc5+ Kf1 Ng3+ Ke1 Bb4+ Kd1 Bb3+ ' +
      'Kc1 Ne2+ Kb1 Nc3+ Kc1 Rc2#'
    ).split(' ')
  };

  var KN = [[1, 2], [2, 1], [-1, 2], [-2, 1], [1, -2], [2, -1], [-1, -2], [-2, -1]];
  var KG = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]];
  var DG = [[1, 1], [1, -1], [-1, 1], [-1, -1]];
  var OR = [[1, 0], [-1, 0], [0, 1], [0, -1]];

  function start() {
    var b = new Array(64).fill(null), back = 'rnbqkbnr', f;
    for (f = 0; f < 8; f++) {
      b[f] = { t: back[f], c: 'w' };
      b[8 + f] = { t: 'p', c: 'w' };
      b[48 + f] = { t: 'p', c: 'b' };
      b[56 + f] = { t: back[f], c: 'b' };
    }
    return b;
  }

  function on(f, r) { return f >= 0 && f < 8 && r >= 0 && r < 8; }

  function attacked(b, sq, by) {
    var f = sq & 7, r = sq >> 3, i, d, p, nf, nr;
    var pr = by === 'w' ? r - 1 : r + 1;
    for (i = -1; i <= 1; i += 2) {
      if (on(f + i, pr)) { p = b[pr * 8 + f + i]; if (p && p.c === by && p.t === 'p') return true; }
    }
    for (i = 0; i < 8; i++) {
      nf = f + KN[i][0]; nr = r + KN[i][1];
      if (on(nf, nr)) { p = b[nr * 8 + nf]; if (p && p.c === by && p.t === 'n') return true; }
      nf = f + KG[i][0]; nr = r + KG[i][1];
      if (on(nf, nr)) { p = b[nr * 8 + nf]; if (p && p.c === by && p.t === 'k') return true; }
    }
    function ray(dirs, kinds) {
      for (var k = 0; k < dirs.length; k++) {
        var x = f + dirs[k][0], y = r + dirs[k][1];
        while (on(x, y)) {
          var q = b[y * 8 + x];
          if (q) { if (q.c === by && kinds.indexOf(q.t) >= 0) return true; break; }
          x += dirs[k][0]; y += dirs[k][1];
        }
      }
      return false;
    }
    return ray(DG, 'bq') || ray(OR, 'rq');
  }

  function targets(b, from) {
    var p = b[from], f = from & 7, r = from >> 3, out = [], i, nf, nr, q, dirs, k, x, y;
    if (p.t === 'n' || p.t === 'k') {
      var set = p.t === 'n' ? KN : KG;
      for (i = 0; i < 8; i++) {
        nf = f + set[i][0]; nr = r + set[i][1];
        if (!on(nf, nr)) continue;
        q = b[nr * 8 + nf];
        if (!q || q.c !== p.c) out.push(nr * 8 + nf);
      }
      return out;
    }
    dirs = p.t === 'b' ? DG : p.t === 'r' ? OR : DG.concat(OR);
    for (k = 0; k < dirs.length; k++) {
      x = f + dirs[k][0]; y = r + dirs[k][1];
      while (on(x, y)) {
        q = b[y * 8 + x];
        if (!q) out.push(y * 8 + x);
        else { if (q.c !== p.c) out.push(y * 8 + x); break; }
        x += dirs[k][0]; y += dirs[k][1];
      }
    }
    return out;
  }

  function kingSq(b, c) {
    for (var i = 0; i < 64; i++) if (b[i] && b[i].t === 'k' && b[i].c === c) return i;
    return -1;
  }

  function legalAfter(b, from, to, c) {
    var n = b.slice();
    n[to] = n[from]; n[from] = null;
    return !attacked(n, kingSq(n, c), c === 'w' ? 'b' : 'w');
  }

  /** Resolve a SAN string into a concrete move on board `b`. */
  function resolve(b, san, c) {
    var s = san.replace(/[+#!?]/g, ''), rank, kind, i;
    if (s === 'O-O' || s === 'O-O-O') {
      rank = c === 'w' ? 0 : 7;
      kind = s === 'O-O';
      return {
        from: rank * 8 + 4, to: rank * 8 + (kind ? 6 : 2),
        rook: [rank * 8 + (kind ? 7 : 0), rank * 8 + (kind ? 5 : 3)], piece: 'k'
      };
    }
    var piece = 'p';
    if ('NBRQK'.indexOf(s[0]) >= 0) { piece = s[0].toLowerCase(); s = s.slice(1); }
    var capture = s.indexOf('x') >= 0;
    s = s.replace('x', '');
    var tf = s.charCodeAt(s.length - 2) - 97, tr = +s[s.length - 1] - 1, to = tr * 8 + tf;
    var dis = s.slice(0, -2), found = [];
    var dir = c === 'w' ? 1 : -1;

    if (piece === 'p') {
      var ff = capture ? dis.charCodeAt(0) - 97 : tf;
      var r1 = tr - dir, from = r1 * 8 + ff, p = b[from];
      if (!capture && !(p && p.t === 'p' && p.c === c)) {
        var r2 = tr - 2 * dir;
        from = r2 * 8 + ff; p = b[from];
      }
      return { from: from, to: to, piece: 'p' };
    }
    for (i = 0; i < 64; i++) {
      var q = b[i];
      if (!q || q.c !== c || q.t !== piece) continue;
      if (targets(b, i).indexOf(to) < 0) continue;
      if (dis) {
        var d = dis[0];
        if (d >= 'a' && d <= 'h') { if ((i & 7) !== d.charCodeAt(0) - 97) continue; }
        else if ((i >> 3) !== +d - 1) continue;
      }
      if (!legalAfter(b, i, to, c)) continue;
      found.push(i);
    }
    if (found.length !== 1) throw new Error('Ambiguous or illegal SAN: ' + san + ' (' + found.length + ')');
    return { from: found[0], to: to, piece: piece };
  }

  /** Apply a resolved move; returns { board, captured } without mutating `b`. */
  function apply(b, m) {
    var n = b.slice(), captured = n[m.to] ? m.to : -1;
    n[m.to] = n[m.from]; n[m.from] = null;
    if (m.rook) { n[m.rook[1]] = n[m.rook[0]]; n[m.rook[0]] = null; }
    return { board: n, captured: captured };
  }

  /** Pre-compute every ply: resolved move, SAN, who moved, and whether it gives check. */
  function plan(game) {
    var b = start(), out = [], i, c, m, r;
    for (i = 0; i < game.moves.length; i++) {
      c = i % 2 === 0 ? 'w' : 'b';
      m = resolve(b, game.moves[i], c);
      r = apply(b, m);
      b = r.board;
      var foe = c === 'w' ? 'b' : 'w', ks = kingSq(b, foe);
      out.push({
        san: game.moves[i], color: c, from: m.from, to: m.to, rook: m.rook || null,
        captured: r.captured, piece: m.piece,
        check: attacked(b, ks, c) ? ks : -1,
        mate: /#$/.test(game.moves[i]),
        number: Math.floor(i / 2) + 1
      });
    }
    return out;
  }

  var api = { GAME: GAME, start: start, plan: plan };
  if (typeof module !== 'undefined' && module.exports) module.exports = api;
  root.ZBraChess = api;
})(typeof window !== 'undefined' ? window : globalThis);