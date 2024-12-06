import { UIBoard, drawArrow } from "./ui_board.js";
import init, { Board } from "../pkg/tangle_rs.js";

async function run() {

    await init();

    let board = new Board("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
    let ui_board = new UIBoard(board);
    // board.legal_moves;
    ui_board.draw();
    // game.doMoves(board.send());
    // readFEN("p7/8/8/8/8/8/8/P7 w KQkq - 0 1");

    document.getElementById('flip_button').addEventListener("click", () => {
        console.log("flip");
        ui_board.flip();
        ui_board.drawBoardCanvas();
        ui_board.clearArrows();
        ui_board.drawArrows();
    });

    window.globs = { board: board, uiboard: ui_board}
}

run();


