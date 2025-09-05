import { useEffect, useMemo, useState } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Label } from "@/components/ui/label";
import { ScrollArea } from "@/components/ui/scroll-area";
import { 
  ArrowLeft, 
  Plane, 
  ShoppingCart, 
  Fuel, 
  Package, 
  Users,
  DollarSign,
  MapPin
} from "lucide-react";
import { listModels, buyPlane } from "@/api/game";

interface Aircraft {
  id: string;
  model: string;
  manufacturer: string;
  price: number;
  maxRange: number;
  cargoCapacity: number;
  fuelCapacity: number;
  passengerCapacity: number;
  cruiseSpeed: number;
  fuelEfficiency: number;
  category: 'light' | 'medium' | 'heavy' | 'cargo';
}

interface Airport {
  id: string;
  name: string;
  code: string;
}

interface AirplanePurchaseScreenProps {
  onBack: () => void;
  onPurchase: (aircraftId: string, airportId: string) => void;
  playerCash: number;
  airportsData?: { id: number; name: string }[];
}

export const AirplanePurchaseScreen = ({ onBack, onPurchase, playerCash, airportsData }: AirplanePurchaseScreenProps) => {
  const [selectedAircraft, setSelectedAircraft] = useState<string>("");
  const [selectedAirport, setSelectedAirport] = useState<string>("");
  const [filterCategory, setFilterCategory] = useState<string>("all");

  const [availableAircraft, setAvailableAircraft] = useState<Aircraft[]>([]);
  const airports: Airport[] = useMemo(() => (
    (airportsData ?? []).map(a => ({ id: String(a.id), code: String(a.id), name: a.name }))
  ), [airportsData]);

  useEffect(() => {
    async function fetch() {
      const models = await listModels();
      setAvailableAircraft(models.map(m => ({
        id: m.name,
        model: m.name,
        manufacturer: '',
        price: Math.round(m.purchase_price),
        maxRange: 0,
        cargoCapacity: Math.round(m.payload_capacity),
        fuelCapacity: Math.round(m.fuel_capacity),
        passengerCapacity: 0,
        cruiseSpeed: Math.round(m.cruise_speed),
        fuelEfficiency: m.fuel_consumption,
        category: m.payload_capacity > 80000 ? 'cargo' : m.payload_capacity > 20000 ? 'heavy' : m.payload_capacity > 5000 ? 'medium' : 'light',
      })));
    }
    fetch();
  }, []);

  const filteredAircraft = availableAircraft.filter(aircraft => 
    filterCategory === "all" || aircraft.category === filterCategory
  );

  const selectedAircraftData = availableAircraft.find(a => a.id === selectedAircraft);
  const canAfford = selectedAircraftData ? playerCash >= selectedAircraftData.price : false;

  const getCategoryColor = (category: Aircraft['category']) => {
    switch (category) {
      case 'light':
        return 'bg-green-500/20 text-green-400 border-green-500/30';
      case 'medium':
        return 'bg-aviation-blue/20 text-aviation-blue border-aviation-blue/30';
      case 'heavy':
        return 'bg-aviation-amber/20 text-aviation-amber border-aviation-amber/30';
      case 'cargo':
        return 'bg-aviation-radar/20 text-aviation-radar border-aviation-radar/30';
      default:
        return 'bg-slate-500/20 text-slate-400 border-slate-500/30';
    }
  };

  const getCategoryIcon = (category: Aircraft['category']) => {
    switch (category) {
      case 'light':
        return '🛩️';
      case 'medium':
        return '✈️';
      case 'heavy':
        return '🛫';
      case 'cargo':
        return '📦';
      default:
        return '✈️';
    }
  };

  return (
    <div className="min-h-screen bg-gradient-control p-4">
      <div className="max-w-7xl mx-auto space-y-4">
        
        {/* Header */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <Button variant="control" onClick={onBack}>
              <ArrowLeft className="w-4 h-4 mr-2" />
              Back
            </Button>
            <div className="flex items-center gap-2">
              <div className="p-2 rounded-lg bg-aviation-blue/20 border border-aviation-blue/30">
                <ShoppingCart className="w-5 h-5 text-aviation-blue" />
              </div>
              <h1 className="text-2xl font-bold text-foreground">Aircraft Marketplace</h1>
            </div>
          </div>
          
          <Badge variant="outline" className="bg-aviation-radar/10 border-aviation-radar/30 text-lg px-4 py-2">
            <DollarSign className="w-4 h-4 mr-1" />
            ${playerCash.toLocaleString()}
          </Badge>
        </div>

        <div className="grid grid-cols-12 gap-4">
          
          {/* Left Panel - Aircraft Selection */}
          <div className="col-span-8 space-y-4">
            
            {/* Category Filter */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">Filter by Category</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="flex gap-2">
                  <Button 
                    variant={filterCategory === "all" ? "runway" : "control"} 
                    size="sm"
                    onClick={() => setFilterCategory("all")}
                  >
                    All Aircraft
                  </Button>
                  <Button 
                    variant={filterCategory === "light" ? "runway" : "control"} 
                    size="sm"
                    onClick={() => setFilterCategory("light")}
                  >
                    🛩️ Light
                  </Button>
                  <Button 
                    variant={filterCategory === "medium" ? "runway" : "control"} 
                    size="sm"
                    onClick={() => setFilterCategory("medium")}
                  >
                    ✈️ Medium
                  </Button>
                  <Button 
                    variant={filterCategory === "heavy" ? "runway" : "control"} 
                    size="sm"
                    onClick={() => setFilterCategory("heavy")}
                  >
                    🛫 Heavy
                  </Button>
                  <Button 
                    variant={filterCategory === "cargo" ? "runway" : "control"} 
                    size="sm"
                    onClick={() => setFilterCategory("cargo")}
                  >
                    📦 Cargo
                  </Button>
                </div>
              </CardContent>
            </Card>

            {/* Aircraft List */}
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">
                  Available Aircraft ({filteredAircraft.length})
                </CardTitle>
              </CardHeader>
              <CardContent>
                <ScrollArea className="h-96">
                  <div className="space-y-4">
                    {filteredAircraft.map((aircraft) => (
                      <div 
                        key={aircraft.id} 
                        className={`border rounded-lg p-4 cursor-pointer transition-all ${
                          selectedAircraft === aircraft.id 
                            ? 'border-aviation-blue bg-aviation-blue/10' 
                            : 'border-aviation-blue/20 bg-secondary/20 hover:bg-secondary/30'
                        } ${playerCash < aircraft.price ? 'opacity-60' : ''}`}
                        onClick={() => setSelectedAircraft(aircraft.id)}
                      >
                        <div className="flex items-start justify-between">
                          <div className="space-y-3 flex-1">
                            <div className="flex items-center gap-3">
                              <span className="text-2xl">{getCategoryIcon(aircraft.category)}</span>
                              <div>
                                <div className="font-semibold text-lg">{aircraft.model}</div>
                                <div className="text-sm text-muted-foreground">{aircraft.manufacturer}</div>
                              </div>
                              <Badge variant="outline" className={getCategoryColor(aircraft.category)}>
                                {aircraft.category}
                              </Badge>
                              {playerCash < aircraft.price && (
                                <Badge variant="destructive">
                                  Cannot Afford
                                </Badge>
                              )}
                            </div>
                            
                            <div className="grid grid-cols-3 gap-4 text-sm">
                              <div className="flex items-center gap-1">
                                <MapPin className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Range:</span>
                                <span className="font-medium">{aircraft.maxRange} km</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Package className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Cargo:</span>
                                <span className="font-medium">{aircraft.cargoCapacity} kg</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Fuel className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Efficiency:</span>
                                <span className="font-medium">{aircraft.fuelEfficiency} km/L</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Plane className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Speed:</span>
                                <span className="font-medium">{aircraft.cruiseSpeed} km/h</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Users className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Passengers:</span>
                                <span className="font-medium">{aircraft.passengerCapacity || 'Cargo Only'}</span>
                              </div>
                              
                              <div className="flex items-center gap-1">
                                <Fuel className="w-3 h-3 text-muted-foreground" />
                                <span className="text-muted-foreground">Fuel Cap:</span>
                                <span className="font-medium">{aircraft.fuelCapacity} L</span>
                              </div>
                            </div>
                          </div>
                          
                          <div className="text-right">
                            <div className="text-aviation-radar font-bold text-2xl">
                              ${aircraft.price.toLocaleString()}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </ScrollArea>
              </CardContent>
            </Card>
          </div>

          {/* Right Panel - Purchase */}
          <div className="col-span-4 space-y-4">
            <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel">
              <CardHeader className="pb-3">
                <CardTitle className="text-aviation-blue">Purchase Aircraft</CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                
                {selectedAircraftData && (
                  <div className="space-y-4">
                    <div className="p-4 bg-secondary/20 rounded-lg">
                      <div className="font-semibold text-lg">{selectedAircraftData.model}</div>
                      <div className="text-aviation-radar font-bold text-xl">
                        ${selectedAircraftData.price.toLocaleString()}
                      </div>
                    </div>

                    <div className="space-y-2">
                      <Label>Starting Airport</Label>
                      <Select onValueChange={setSelectedAirport}>
                        <SelectTrigger className="bg-secondary/50 border-aviation-blue/20">
                          <SelectValue placeholder="Select airport" />
                        </SelectTrigger>
                        <SelectContent>
                          {airports.map((airport) => (
                            <SelectItem key={airport.id} value={airport.id}>
                              {airport.code} - {airport.name}
                            </SelectItem>
                          ))}
                        </SelectContent>
                      </Select>
                    </div>

                    <div className="space-y-2 text-sm">
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Current Cash:</span>
                        <span>${playerCash.toLocaleString()}</span>
                      </div>
                      <div className="flex justify-between">
                        <span className="text-muted-foreground">Aircraft Cost:</span>
                        <span>-${selectedAircraftData.price.toLocaleString()}</span>
                      </div>
                      <hr className="border-aviation-blue/20" />
                      <div className="flex justify-between font-semibold">
                        <span>Remaining Cash:</span>
                        <span className={playerCash - selectedAircraftData.price < 0 ? 'text-red-400' : 'text-aviation-radar'}>
                          ${(playerCash - selectedAircraftData.price).toLocaleString()}
                        </span>
                      </div>
                    </div>

                    <Button 
                      className="w-full" 
                      variant="runway"
                      disabled={!selectedAirport || !canAfford}
                      onClick={async () => {
                        if (!selectedAirport) return;
                        await buyPlane(selectedAircraft, parseInt(selectedAirport, 10));
                        onPurchase(selectedAircraft, selectedAirport);
                      }}
                    >
                      <ShoppingCart className="w-4 h-4 mr-2" />
                      Purchase Aircraft
                    </Button>

                    {!canAfford && (
                      <div className="text-sm text-red-400 text-center">
                        Insufficient funds
                      </div>
                    )}
                  </div>
                )}

                {!selectedAircraftData && (
                  <div className="text-center text-muted-foreground py-8">
                    Select an aircraft to purchase
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </div>
  );
};
