/*!
 * ZBra docs theme — opening sequence.
 * Replays "The Game of the Century" (Byrne – Fischer, 1956) when the book is first opened.
 *
 * Configure before this script loads (optional):
 *   window.ZBRA_INTRO = { frequency: 'session' | 'always' | 'never' };
 * Force it once by appending ?zbra-intro to any URL.
 */
(function () {
  'use strict';

  var CFG = Object.assign({ frequency: 'session' }, window.ZBRA_INTRO || {});
  var KEY = 'zbra-intro-seen';
  var GLYPH = { k: '♚', q: '♛', r: '♜', b: '♝', n: '♞', p: '♟' };
  var TEXT = '\uFE0E'; // force text (non-emoji) presentation for the pawn glyph

  var THEME_NAMES = {
    light: 'Ivory', rust: 'Parchment', coal: 'Obsidian', navy: 'Midnight', ayu: 'Emerald'
  };

  /* Moments worth pausing on. Keys are 1-based plies. */
  var NOTES = {
    1: ['Rosenwald Memorial, 1956', 'Donald Byrne, White. Bobby Fischer, Black, thirteen years old.'],
    33: ['17. Kf1', 'Black’s queen is attacked by the bishop on c5. Any sensible player retreats it.'],
    34: ['17… Be6!!', 'Fischer ignores the queen and offers it again. If White takes, a chain of checks begins.'],
    35: ['18. Bxb6', 'White accepts the queen.'],
    36: ['18… Bxc4+', 'The bishop captures with check, and White’s king is driven back and forth.'],
    38: ['…Ne2+, …Nxd4+, …Nc3+', 'The knight keeps giving check while the bishop on c4 stays out of reach.'],
    46: ['23… axb6', 'The pawn takes the bishop. White’s queen attacks b6 but cannot hold everything.'],
    50: ['25… Nxd1', 'The queen is repaid: a rook, two bishops and a pawn, with the white king still exposed.'],
    82: ['41… Rc2#', 'Mate. The game is remembered as “The Game of the Century”.']
  };
  var ANNOT = { 34: '!!' };

  function timing(p) {
    if (p <= 32) return { dur: 130, wait: 85 };
    var t = {
      33: { dur: 450, wait: 1300 }, 34: { dur: 900, wait: 3800 }, 35: { dur: 550, wait: 1500 },
      36: { dur: 550, wait: 2000 }, 38: { dur: 400, wait: 1700 }, 46: { dur: 400, wait: 1200 },
      50: { dur: 500, wait: 2400 }, 82: { dur: 600, wait: 3600 }
    };
    return t[p] || { dur: 320, wait: 290 };
  }

  /* ------------------------------------------------------------------ */
  /* Theme labels                                                        */
  /* ------------------------------------------------------------------ */
  function renameThemes() {
    Object.keys(THEME_NAMES).forEach(function (id) {
      var el = document.getElementById(id);
      if (el && el.classList.contains('theme')) el.textContent = THEME_NAMES[id];
    });
  }

  /* ------------------------------------------------------------------ */
  /* Intro                                                               */
  /* ------------------------------------------------------------------ */
  var running = null;

  function el(tag, cls, text) {
    var n = document.createElement(tag);
    if (cls) n.className = cls;
    if (text != null) n.textContent = text;
    return n;
  }

  function play() {
    if (running) return;
    var chess = window.ZBraChess;
    if (!chess) return;
    var plies = chess.plan(chess.GAME), game = chess.GAME;

    var aborted = false, html = document.documentElement;
    html.classList.add('zi-lock');

    var root = el('div', 'zi');
    root.setAttribute('role', 'dialog');
    root.setAttribute('aria-label', 'ZBra introduction: Byrne versus Fischer, 1956');

    /* title card */
    var card = el('div', 'zi-card');
    card.appendChild(el('div', 'zi-wordmark', 'ZBra'));
    card.appendChild(el('div', 'zi-sub', 'A Redesigned Chess Engine, By Luminx Labs'));
    root.appendChild(card);

    /* stage */
    var stage = el('div', 'zi-stage zi-hidden');
    var boardWrap = el('div', 'zi-boardwrap');
    var board = el('div', 'zi-board');
    var sq = [], pcs = new Array(64).fill(null), s, x, y, f, r;

    for (y = 0; y < 8; y++) {
      for (x = 0; x < 8; x++) {
        f = 7 - x; r = y; s = r * 8 + f; // Black at the bottom: Fischer's view
        var cell = el('div', 'zi-sq ' + ((f + r) % 2 === 1 ? 'zi-lt' : 'zi-dk'));
        if (x === 0) cell.setAttribute('data-rank', String(r + 1));
        if (y === 7) cell.setAttribute('data-file', 'abcdefgh'[f]);
        sq[s] = cell; board.appendChild(cell);
      }
    }

    function put(piece, at) {
      pcs[at] = piece;
      piece.style.setProperty('--x', 7 - (at & 7));
      piece.style.setProperty('--y', at >> 3);
    }
    var start = chess.start();
    start.forEach(function (p, i) {
      if (!p) return;
      var n = el('div', 'zi-p ' + (p.c === 'w' ? 'zi-w' : 'zi-b'), GLYPH[p.t] + TEXT);
      board.appendChild(n); put(n, i);
    });

    var badge = el('div', 'zi-badge');
    badge.appendChild(el('span', null, '!!'));
    badge.hidden = true;
    board.appendChild(badge);
    boardWrap.appendChild(board);
    stage.appendChild(boardWrap);

    /* side panel */
    var side = el('aside', 'zi-side');
    side.appendChild(el('p', 'zi-players', 'Byrne – Fischer'));
    side.appendChild(el('p', 'zi-event', game.event + ', ' + game.place + ', ' + game.date));
    var headline = el('h2', 'zi-headline', '');
    var body = el('p', 'zi-body', '');
    body.setAttribute('aria-live', 'polite');
    var list = el('div', 'zi-list');
    side.appendChild(headline); side.appendChild(body); side.appendChild(list);
    stage.appendChild(side);
    root.appendChild(stage);

    /* footer */
    var foot = el('div', 'zi-foot');
    var prog = el('div', 'zi-progress'), bar = el('i');
    prog.appendChild(bar);
    var skip = el('button', 'zi-skip', 'Skip to the docs');
    skip.type = 'button';
    foot.appendChild(prog); foot.appendChild(skip);
    root.appendChild(foot);

    document.body.appendChild(root);
    skip.focus({ preventScroll: true });

    function wait(ms) { return new Promise(function (res) { setTimeout(res, ms); }); }

    function finish() {
      if (!running) return;
      aborted = true;
      try { sessionStorage.setItem(KEY, '1'); } catch (e) { /* storage may be blocked */ }
      document.removeEventListener('keydown', onKey);
      root.classList.add('zi-out');
      setTimeout(function () {
        root.remove(); html.classList.remove('zi-lock'); running = null;
      }, 900);
    }
    function onKey(e) { if (e.key === 'Escape') finish(); }
    skip.addEventListener('click', finish);
    document.addEventListener('keydown', onKey);
    running = { finish: finish };

    function clearMarks() {
      for (var i = 0; i < 64; i++) sq[i].classList.remove('zi-last', 'zi-chk', 'zi-hot');
      badge.hidden = true;
    }

    function note(p) {
      var n = NOTES[p];
      if (!n) return;
      headline.textContent = n[0];
      body.textContent = n[1];
      headline.classList.remove('zi-pop'); void headline.offsetWidth; headline.classList.add('zi-pop');
    }

    function logPly(m, p) {
      var last = list.lastElementChild;
      if (m.color === 'w' || !last) {
        last = el('span', 'zi-turn');
        last.appendChild(el('b', null, m.number + '.'));
        list.appendChild(last);
      }
      last.appendChild(el('span', null, m.san + (ANNOT[p] || '')));
      list.scrollTop = list.scrollHeight;
    }

    async function move(m, p) {
      var t = timing(p);
      board.style.setProperty('--dur', t.dur + 'ms');
      clearMarks();
      if (m.captured >= 0 && pcs[m.captured]) {
        var gone = pcs[m.captured];
        gone.classList.add('zi-taken');
        setTimeout(function () { gone.remove(); }, 500);
        pcs[m.captured] = null;
      }
      var mover = pcs[m.from]; pcs[m.from] = null; put(mover, m.to);
      if (m.rook) { var rk = pcs[m.rook[0]]; pcs[m.rook[0]] = null; put(rk, m.rook[1]); }

      sq[m.from].classList.add('zi-last'); sq[m.to].classList.add('zi-last');
      if (m.check >= 0) sq[m.check].classList.add('zi-chk');
      if (p === 34) {
        sq[5 * 8 + 1].classList.add('zi-hot');     // the queen on b6
        badge.style.setProperty('--x', 7 - (m.to & 7));
        badge.style.setProperty('--y', m.to >> 3);
        badge.hidden = false;
      }
      stage.classList.toggle('zi-spot', p === 34);
      note(p); logPly(m, p);
      bar.style.width = (p / plies.length * 100) + '%';
      await wait(t.wait);
    }

    (async function run() {
      await wait(2300);
      if (aborted) return;
      card.classList.add('zi-card-out');
      stage.classList.remove('zi-hidden');
      await wait(1100);
      for (var p = 1; p <= plies.length; p++) {
        if (aborted) return;
        await move(plies[p - 1], p);
      }
      finish();
    })();
  }

  /* ------------------------------------------------------------------ */
  /* Boot                                                                */
  /* ------------------------------------------------------------------ */
  function boot() {
    renameThemes();

    var rb = document.querySelector('#menu-bar .right-buttons');
    if (rb && !document.getElementById('zbra-replay')) {
      var b = el('button', 'icon-button zbra-replay', '♞' + TEXT);
      b.id = 'zbra-replay'; b.type = 'button';
      b.title = 'Replay Fischer’s brilliancy';
      b.setAttribute('aria-label', 'Replay Fischer’s brilliancy');
      b.addEventListener('click', play);
      rb.insertBefore(b, rb.firstChild);
    }

    var forced = /[?&]zbra-intro\b/.test(location.search);
    var seen = false;
    try { seen = sessionStorage.getItem(KEY) === '1'; } catch (e) { /* ignore */ }
    var reduce = window.matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches;

    if (forced) return play();
    if (CFG.frequency === 'never' || reduce) return;
    if (CFG.frequency === 'always' || !seen) play();
  }

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', boot);
  else boot();
})();
