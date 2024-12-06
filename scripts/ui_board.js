const SQUARE_SIZE = 80;
const PIECE_SIZE = 70;
const TILE_OFFSET = (SQUARE_SIZE - PIECE_SIZE) / 2;
const getTilesRect = () => document.getElementById("tiles").getBoundingClientRect()
const idxToSquare = (idx) => { return { row: Math.floor(idx / 8), col: idx % 8 } }

export class UIBoard {
    constructor(state) {
        this.squares = [];
        this.flipped = false;
        this.chessBoard = document.getElementById("chessboard");
        this.state = state;
    }
    draw() {
        this.drawBoardCanvas();
        JSON.parse(this.state.send_board()).map(([square, piece]) => this.addPiece(piece.kind, piece.color, square));
        this.drawArrows();
    }
    get(idx) {
        return this.squares[idx];
    }
    put(piece, to) {
        this.removePiece(to);
        this.squares[to] = piece;
        moveToSquare(piece, to);
    }
    move(from, to) {
        let piece = this.get(from);
        this.put(piece, to);
        delete this.squares[from];
    }
    flip() {
        let orig_squares = this.squares;
        this.squares = {};
        for (let i in orig_squares) {
            let piece = orig_squares[i];
            this.put(piece, 63 - i);
        }
        this.flipped = !this.flipped;
    }
    addPiece(kind, color, square) {
        let piece = document.createElement("img");
        piece.src = "assets/" + kind + "_" + color + ".svg";
        piece.classList.add("piece", color, kind);
        piece.addEventListener("mousedown", (e) => { this.dragPiece(e) });
        piece.ondragstart = function() { return false };
        this.put(piece, square);
        this.chessBoard.appendChild(piece);
    }
    removePiece(square) {
        let piece = this.get(square);
        if (piece) {
            this.chessBoard.removeChild(piece);
            delete this.squares[square];
        }

    }
    drawBoardCanvas() {
        const flipped = this.flipped;
        const files = ["a", "b", "c", "d", "e", "f", "g", "h"];
        const ranks = ["8", "7", "6", "5", "4", "3", "2", "1"]
        const tiles = document.getElementById("tiles");
        tiles.innerHTML = "";
        const f1 = flipped ? 7 : 0;
        const f2 = flipped ? 1 : -1;
        for (let i = 0; i < 64; i++) {
            let tile = document.createElement("div");
            tile.classList.add("tile");
            tile.classList.add((Math.floor(i / 8) + i) % 2 === 0 ? "light" : "dark");
            if (i % 8 == 0) {
                let label = document.createElement("div");
                label.classList.add("label", "number");
                label.textContent = ranks[f1 - (i / 8) * f2];
                tile.appendChild(label)
            }
            if (i > (64 - 8) - 1) {
                let label = document.createElement("div");
                label.classList.add("label", "letter");
                label.textContent = files[f1 - (i - (64 - 8)) * f2];
                tile.appendChild(label);
            }
            tiles.appendChild(tile);
        }
    }

    dragPiece(e) {
        const piece = e.target;
        const orig_square = getSquareUnderCursor(e.clientX, e.clientY);
        const tilesRect = getTilesRect();

        piece.style.zIndex = "100";
        piece.style.width = PIECE_SIZE * 1.25 + "px";
        piece.style.height = PIECE_SIZE * 1.25 + "px";
        piece.style.left = parseInt(piece.style.left) - PIECE_SIZE * .25 / 2 + "px";
        piece.style.top = parseInt(piece.style.top) - PIECE_SIZE * .25 / 2 + "px";

        const pieceRect = piece.getBoundingClientRect();
        const shiftX = e.clientX - pieceRect.left;
        const shiftY = e.clientY - pieceRect.top;

        function onMouseMove(e) {
            piece.style.left = e.clientX - tilesRect.left - shiftX + "px";
            piece.style.top = e.clientY - tilesRect.top - shiftY + "px";
        }

        function onMouseUp(e, obj) {
            piece.style.zIndex = "10";
            piece.style.width = PIECE_SIZE + "px";
            piece.style.height = PIECE_SIZE + "px";

            document.removeEventListener("mousemove", onMouseMove);
            const target_square = getSquareUnderCursor(e.clientX, e.clientY);

            function abort() {
                moveToSquare(piece, orig_square);
                showAvailableMoves(obj.legal_moves[orig_square]);
            }

            if ((target_square !== null) && (orig_square !== target_square)) {
                if (obj.isLegalMove(orig_square, target_square) && obj.makeMove(orig_square, target_square)) {
                    hideAvailableMoves(); return
                }
            }
            abort();
        };

        document.addEventListener("mousemove", onMouseMove);
        document.addEventListener("mouseup", (e) => { onMouseUp(e, this) }, { once: true });
        hideAvailableMoves();
    }

    async makeMove(from, to) {
        let promotion = await this.may_promote(from, to);
        if (this.flipped) {
            from = 63 - from;
            to = 63 - to;
        }
        let [ok, response] = JSON.parse(this.state.input_move(from, to, promotion));
        if (!ok) { return false };
        for (let update of response.updates) {
            let [f, t] = update;
            if (this.flipped) {
                [f, t] = [63 - f, t < 0 ? t : 63 - t];
            }
            if (t < 0) {
                this.removePiece(f);
            } else {
                this.move(f, t);
            }
        }
        if (promotion > 0) {
            let color = this.get(to).classList[1];
            switch (promotion) {
                case 1:
                    this.addPiece("queen", color, to);
                    break;
                case 2:
                    this.addPiece("rook", color, to);
                    break;
                case 3:
                    this.addPiece("bishop", color, to);
                    break;
                case 4:
                    this.addPiece("knight", color, to);
                    break;
            }
        }
        this.clearArrows()
        this.drawArrows()
        return true
    }

    isLegalMove(from, to) {
        let moves = this.legal_moves[from];
        if (!moves) { return false }
        return this.legal_moves[from].includes(to)
    }

    drawArrows() {
        let moves = this.legal_moves;
        for (let from in moves) {
            for (let to of moves[from]) {
                drawArrow(from, to);
            }
        }
    }
    clearArrows() {
        const arrowsContainer = this.chessBoard.querySelector("svg.arrows-container");
        if (arrowsContainer) {
            arrowsContainer.innerHTML = ''; // Remove all child elements
        }
    }
    async may_promote(from, to) {
        let piece = this.get(from);
        if (piece.classList.contains("pawn") && (to < 8 || to > 56)) {
            const choice = await promotion_dialog(piece.classList[1]);
            return choice
        };
        return 0;
    }
    get legal_moves() {
        let moves = this.state.get_legal_moves();
        if (this.flipped) {
            let flipped_moves = [];
            for (let i in moves) {
                let m = [];
                for (let move of moves[i]) {
                    m.push(63 - move);
                }
                flipped_moves[63 - i] = m;
            }
            return flipped_moves
        }
        return moves;
    }
}

function promotion_dialog(color) {
    return new Promise((resolve) => {
        const dialog = document.createElement('div');
        dialog.style.position = 'fixed';
        dialog.style.top = '50%';
        dialog.style.left = '50%';
        dialog.style.transform = 'translate(-50%, -50%)';
        dialog.style.backgroundColor = '#fff';
        dialog.style.padding = '20px';
        dialog.style.border = '1px solid #ccc';
        dialog.style.boxShadow = '0 2px 10px rgba(0, 0, 0, 0.2)';
        dialog.style.zIndex = '1000';
        dialog.id = "promotion_dialog";

        let board = document.getElementById("chessboard");
        board.appendChild(dialog);

        let queen = document.createElement("img")
        queen.src = "assets/" + "queen" + "_" + color + ".svg";
        queen.addEventListener("click", () => { board.removeChild(dialog), resolve(1) })

        let rook = document.createElement("img")
        rook.src = "assets/" + "rook" + "_" + color + ".svg";
        rook.addEventListener("click", () => { board.removeChild(dialog); resolve(2) })

        let bishop = document.createElement("img")
        bishop.src = "assets/" + "bishop" + "_" + color + ".svg";
        bishop.addEventListener("click", () => { board.removeChild(dialog), resolve(3) })

        let knight = document.createElement("img")
        knight.src = "assets/" + "knight" + "_" + color + ".svg";
        knight.addEventListener("click", () => { board.removeChild(dialog); resolve(4) })

        dialog.appendChild(queen);
        dialog.appendChild(rook);
        dialog.appendChild(bishop);
        dialog.appendChild(knight);
    });
}

function getSquareUnderCursor(x, y) {
    const tilesRect = getTilesRect();
    if (x < tilesRect.left || x > tilesRect.right || y < tilesRect.top || y > tilesRect.bottom) {
        return
    }
    x -= tilesRect.left;
    y -= tilesRect.top;
    return Math.floor(y / SQUARE_SIZE) * 8 + Math.floor(x / SQUARE_SIZE)
}

function moveToSquare(piece, idx) {
    let { row, col } = idxToSquare(idx);
    piece.style.left = (col * SQUARE_SIZE) + TILE_OFFSET + "px";
    piece.style.top = (row * SQUARE_SIZE) + TILE_OFFSET + "px";
}


function showAvailableMoves(moves) {
    if (!moves) { return }
    let tiles = document.getElementsByClassName("tile");
    for (let i of moves) {
        let tile = tiles[i];
        tile.classList.replace("dark", "dark-move")
        tile.classList.replace("light", "light-move")
    }
}
function hideAvailableMoves() {
    let tiles = document.getElementsByClassName("tile");
    for (let tile of tiles) {
        tile.classList.replace("dark-move", "dark")
        tile.classList.replace("light-move", "light")
    }
}


export function drawArrow(tail, head) {
    function tileCrd(idx) {
        let { row, col } = idxToSquare(idx);
        return { x: col * SQUARE_SIZE + SQUARE_SIZE / 2, y: row * SQUARE_SIZE + SQUARE_SIZE / 2 }
    }
    tail = tileCrd(tail);
    head = tileCrd(head);
    const board = document.getElementById("chessboard");

    // Check if the arrows container SVG exists; if not, create it
    let arrowsContainer = board.querySelector("svg.arrows-container");
    if (!arrowsContainer) {
        arrowsContainer = document.createElementNS("http://www.w3.org/2000/svg", "svg");
        arrowsContainer.classList.add("arrows-container");
        arrowsContainer.setAttribute("width", "100%");
        arrowsContainer.setAttribute("height", "100%");
        arrowsContainer.style.position = "absolute";
        arrowsContainer.style.top = 0;
        arrowsContainer.style.left = 0;
        arrowsContainer.setAttribute("pointer-events", "none")
        board.appendChild(arrowsContainer);
    }

    // Create the line element for the arrow shaft
    const line = document.createElementNS("http://www.w3.org/2000/svg", "line");
    line.setAttribute("x1", tail.x);
    line.setAttribute("y1", tail.y);
    line.setAttribute("x2", head.x);
    line.setAttribute("y2", head.y);
    line.setAttribute("stroke", "black");
    line.setAttribute("stroke-width", "1");
    line.setAttribute("stroke-opacity", "0.4");


    // Create the marker (arrowhead) definition if it doesn't exist
    let marker = arrowsContainer.querySelector("marker#arrowhead");
    if (!marker) {
        marker = document.createElementNS("http://www.w3.org/2000/svg", "marker");
        marker.setAttribute("id", "arrowhead");
        marker.setAttribute("markerWidth", "10");
        marker.setAttribute("markerHeight", "7");
        marker.setAttribute("refX", "10");
        marker.setAttribute("refY", "3.5");
        marker.setAttribute("orient", "auto");
        marker.setAttribute("fill", "black");
        marker.setAttribute("fill-opacity", "0.4");


        // Create the path for the arrowhead shape
        const arrowheadPath = document.createElementNS("http://www.w3.org/2000/svg", "path");
        arrowheadPath.setAttribute("d", "M 0 0 L 10 3.5 L 0 7 z");
        marker.appendChild(arrowheadPath);

        // Append marker to SVG defs
        const defs = document.createElementNS("http://www.w3.org/2000/svg", "defs");
        defs.appendChild(marker);
        arrowsContainer.appendChild(defs);
    }

    // Link marker to the line and append line to arrowsContainer
    line.setAttribute("marker-end", "url(#arrowhead)");
    arrowsContainer.appendChild(line);
}

