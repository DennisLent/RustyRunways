import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Plane, Settings, Play, FolderOpen } from "lucide-react";

interface MainMenuProps {
  onStartGame: (config: GameConfig) => void;
  onLoadGame: (saveName: string) => void;
  onLoadConfig: (file: File) => void;
}

interface GameConfig {
  seed: string;
  airportCount: number;
  startingCash: number;
}

export const MainMenu = ({ onStartGame, onLoadGame, onLoadConfig }: MainMenuProps) => {
  const [gameConfig, setGameConfig] = useState<GameConfig>({
    seed: "",
    airportCount: 10,
    startingCash: 100000,
  });
  const [loadGameName, setLoadGameName] = useState("");
  const [selectedFile, setSelectedFile] = useState<File | null>(null);

  const handleStartGame = () => {
    onStartGame(gameConfig);
  };

  const handleLoadGame = () => {
    if (loadGameName.trim()) {
      onLoadGame(loadGameName);
    }
  };

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
    }
  };

  const handleLoadConfig = () => {
    if (selectedFile) {
      onLoadConfig(selectedFile);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-control flex items-center justify-center p-6">
      <div className="w-full max-w-4xl space-y-8">
        
        {/* Game Title */}
        <div className="text-center space-y-4">
          <div className="flex items-center justify-center gap-3 mb-4">
            <div className="p-3 rounded-xl bg-aviation-blue/20 border border-aviation-blue/30">
              <Plane className="w-8 h-8 text-aviation-blue" />
            </div>
            <h1 className="text-5xl font-bold bg-gradient-sky bg-clip-text text-transparent">
              Rusty Runways
            </h1>
          </div>
          <p className="text-xl text-muted-foreground">
            Build your aviation empire • Manage logistics • Conquer the skies
          </p>
        </div>

        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          
          {/* New Game */}
          <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-control">
            <CardHeader>
              <CardTitle className="flex items-center gap-3 text-aviation-blue">
                <Play className="w-5 h-5" />
                Start New Game
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="seed" className="text-foreground">Game Seed</Label>
                  <Input
                    id="seed"
                    placeholder="Enter seed (optional)"
                    value={gameConfig.seed}
                    onChange={(e) => setGameConfig({ ...gameConfig, seed: e.target.value })}
                    className="bg-secondary/50 border-aviation-blue/20 focus:border-aviation-blue/50"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="airports" className="text-foreground">Number of Airports</Label>
                  <Input
                    id="airports"
                    type="number"
                    min="5"
                    max="50"
                    value={gameConfig.airportCount}
                    onChange={(e) => setGameConfig({ ...gameConfig, airportCount: parseInt(e.target.value) || 10 })}
                    className="bg-secondary/50 border-aviation-blue/20 focus:border-aviation-blue/50"
                  />
                </div>

                <div className="space-y-2">
                  <Label htmlFor="cash" className="text-foreground">Starting Cash ($)</Label>
                  <Input
                    id="cash"
                    type="number"
                    min="10000"
                    max="1000000"
                    step="10000"
                    value={gameConfig.startingCash}
                    onChange={(e) => setGameConfig({ ...gameConfig, startingCash: parseInt(e.target.value) || 100000 })}
                    className="bg-secondary/50 border-aviation-blue/20 focus:border-aviation-blue/50"
                  />
                </div>
              </div>

              <Button 
                onClick={handleStartGame} 
                className="w-full h-12" 
                variant="runway"
                size="lg"
              >
                <Play className="w-5 h-5 mr-2" />
                Launch Game
              </Button>
            </CardContent>
          </Card>

          {/* Load Game */}
          <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-control">
            <CardHeader>
              <CardTitle className="flex items-center gap-3 text-aviation-sky">
                <FolderOpen className="w-5 h-5" />
                Load Saved Game
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="loadGame" className="text-foreground">Save Game Name</Label>
                  <Input
                    id="loadGame"
                    placeholder="Enter save name"
                    value={loadGameName}
                    onChange={(e) => setLoadGameName(e.target.value)}
                    className="bg-secondary/50 border-aviation-blue/20 focus:border-aviation-blue/50"
                  />
                </div>

                {/* Recent Saves */}
                <div className="space-y-2">
                  <Label className="text-muted-foreground text-sm">Recent Saves</Label>
                  <div className="space-y-1">
                    {['campaign_save_1', 'quick_game_2024', 'challenge_mode'].map((save) => (
                      <Button
                        key={save}
                        variant="ghost"
                        size="sm"
                        className="w-full justify-start text-left h-8 text-muted-foreground hover:text-foreground"
                        onClick={() => setLoadGameName(save)}
                      >
                        {save}
                      </Button>
                    ))}
                  </div>
                </div>
              </div>

              <Button 
                onClick={handleLoadGame} 
                disabled={!loadGameName.trim()}
                className="w-full h-12" 
                variant="control"
                size="lg"
              >
                <FolderOpen className="w-5 h-5 mr-2" />
                Load Game
              </Button>
            </CardContent>
          </Card>

          {/* Load Config File */}
          <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-control">
            <CardHeader>
              <CardTitle className="flex items-center gap-3 text-aviation-amber">
                <Settings className="w-5 h-5" />
                Load Config File
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-6">
              
              <div className="space-y-4">
                <div className="space-y-2">
                  <Label htmlFor="configFile" className="text-foreground">Configuration File</Label>
                  <Input
                    id="configFile"
                    type="file"
                    accept=".json,.config"
                    onChange={handleFileSelect}
                    className="bg-secondary/50 border-aviation-blue/20 focus:border-aviation-blue/50 file:bg-aviation-blue/20 file:text-aviation-blue file:border-0 file:rounded-md file:px-3 file:py-1"
                  />
                  {selectedFile && (
                    <p className="text-sm text-muted-foreground">
                      Selected: {selectedFile.name}
                    </p>
                  )}
                </div>

                <div className="text-sm text-muted-foreground space-y-2">
                  <p>Load a pre-configured game setup including:</p>
                  <ul className="list-disc list-inside space-y-1 text-xs">
                    <li>Custom airport layouts</li>
                    <li>Predefined scenarios</li>
                    <li>Challenge configurations</li>
                  </ul>
                </div>
              </div>

              <Button 
                onClick={handleLoadConfig} 
                disabled={!selectedFile}
                className="w-full h-12" 
                variant="warning"
                size="lg"
              >
                <Settings className="w-5 h-5 mr-2" />
                Load Configuration
              </Button>
            </CardContent>
          </Card>
        </div>

        {/* Footer */}
        <div className="text-center">
          <Button variant="ghost" size="sm" className="text-muted-foreground">
            <Settings className="w-4 h-4 mr-2" />
            Settings
          </Button>
        </div>
      </div>
    </div>
  );
};