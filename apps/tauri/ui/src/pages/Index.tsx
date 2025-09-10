import { useState } from "react";
import { MainMenu } from "@/components/MainMenu";
import { GameScreen } from "@/components/GameScreen";
import { newGame, loadGame, startFromConfigYaml } from "@/api/game";

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
      await loadGame(saveName);
      setGameState('playing');
    } catch (e) {
      console.error('Failed to load game', e);
      alert(`Failed to load game: ${e}`);
    }
  };

  const handleLoadConfig = async (file: File) => {
    try {
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
