import { useState } from "react";
import { MainMenu } from "@/components/MainMenu";
import { GameScreen } from "@/components/GameScreen";
import { newGame, loadGame, startFromConfigYaml } from "@/api/game";
import { isTauri } from "@/lib/tauri";

interface GameConfig {
  seed: string;
  airportCount: number;
  startingCash: number;
}

const Index = () => {
  const [gameState, setGameState] = useState<'menu' | 'playing'>('menu');
  const [currentGame, setCurrentGame] = useState<GameConfig | null>(null);

  const handleStartGame = async (config: GameConfig) => {
    try {
      if (!isTauri()) {
        alert(
          "This action requires the Tauri desktop app. From apps/tauri/src-tauri run: 'cargo tauri dev' (it will start the UI and inject Tauri APIs)."
        );
        return;
      }
      await newGame(config.seed, config.airportCount, config.startingCash);
      setCurrentGame(config);
      setGameState('playing');
    } catch (e) {
      console.error('Failed to start game', e);
      alert(`Failed to start game: ${e}`);
    }
  };

  const handleLoadGame = async (saveName: string) => {
    try {
      if (!isTauri()) {
        alert(
          "This action requires the Tauri desktop app. From apps/tauri/src-tauri run: 'cargo tauri dev'."
        );
        return;
      }
      await loadGame(saveName);
      setGameState('playing');
    } catch (e) {
      console.error('Failed to load game', e);
      alert(`Failed to load game: ${e}`);
    }
  };

  const handleLoadConfig = async (file: File) => {
    try {
      if (!isTauri()) {
        alert(
          "This action requires the Tauri desktop app. From apps/tauri/src-tauri run: 'cargo tauri dev'."
        );
        return;
      }
      const text = await file.text();
      await startFromConfigYaml(text);
      setGameState('playing');
    } catch (e) {
      console.error('Failed to load config', e);
      alert(`Failed to load config: ${e}`);
    }
  };

  const handleBackToMenu = () => {
    setGameState('menu');
    setCurrentGame(null);
  };

  if (gameState === 'playing') {
    return <GameScreen onMainMenu={handleBackToMenu} />;
  }

  return (
    <MainMenu 
      onStartGame={handleStartGame}
      onLoadGame={handleLoadGame}
      onLoadConfig={handleLoadConfig}
    />
  );
};

export default Index;
