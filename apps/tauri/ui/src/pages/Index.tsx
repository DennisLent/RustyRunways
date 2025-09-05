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
    await newGame(config.seed, config.airportCount, config.startingCash);
    setCurrentGame(config);
    setGameState('playing');
  };

  const handleLoadGame = async (saveName: string) => {
    await loadGame(saveName);
    setGameState('playing');
  };

  const handleLoadConfig = async (file: File) => {
    const text = await file.text();
    await startFromConfigYaml(text);
    setGameState('playing');
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
