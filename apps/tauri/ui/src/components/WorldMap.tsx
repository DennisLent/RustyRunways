import { useState, useRef, useEffect } from "react";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { ZoomIn, ZoomOut, RotateCcw, MapPin, Plane } from "lucide-react";

interface Airport {
  id: string;
  name: string;
  code: string;
  x: number;
  y: number;
  hasOrders: boolean;
  orderCount: number;
}

interface Airplane {
  id: string;
  model: string;
  x: number;
  y: number;
  status: 'parked' | 'en-route' | 'loading';
  destination?: string;
}

interface WorldMapProps {
  onAirportClick: (airport: Airport) => void;
  onAirplaneClick: (airplane: Airplane) => void;
  airportsData?: Airport[];
  airplanesData?: Airplane[];
  plannedPaths?: { from: { x: number; y: number }, to: { x: number; y: number } }[];
}

export const WorldMap = ({ onAirportClick, onAirplaneClick, airportsData, airplanesData, plannedPaths }: WorldMapProps) => {
  const [zoom, setZoom] = useState(0.5);
  const [pan, setPan] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const mapRef = useRef<HTMLDivElement>(null);

  // Sample data - in real implementation, this would come from props
  const airports: Airport[] = airportsData ?? [
    { id: "JFK", name: "John F. Kennedy", code: "JFK", x: 8500, y: 2000, hasOrders: true, orderCount: 5 },
    { id: "LAX", name: "Los Angeles Intl", code: "LAX", x: 1500, y: 2500, hasOrders: true, orderCount: 3 },
    { id: "DFW", name: "Dallas Fort Worth", code: "DFW", x: 4500, y: 3500, hasOrders: false, orderCount: 0 },
    { id: "MIA", name: "Miami International", code: "MIA", x: 7500, y: 6000, hasOrders: true, orderCount: 2 },
    { id: "SEA", name: "Seattle-Tacoma", code: "SEA", x: 1000, y: 1000, hasOrders: false, orderCount: 0 },
    { id: "DEN", name: "Denver International", code: "DEN", x: 3500, y: 2800, hasOrders: true, orderCount: 4 },
  ];

  const airplanes: Airplane[] = airplanesData ?? [
    { id: "P001", model: "Boeing 737", x: 8500, y: 2000, status: 'parked' },
    { id: "P002", model: "Cessna 172", x: 3000, y: 2200, status: 'en-route', destination: "LAX" },
    { id: "P003", model: "Boeing 777", x: 1500, y: 2500, status: 'loading' },
  ];

  const handleMouseDown = (e: React.MouseEvent) => {
    setIsDragging(true);
    setDragStart({ x: e.clientX - pan.x, y: e.clientY - pan.y });
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    if (isDragging) {
      setPan({
        x: e.clientX - dragStart.x,
        y: e.clientY - dragStart.y,
      });
    }
  };

  const handleMouseUp = () => {
    setIsDragging(false);
  };

  const handleZoomIn = () => {
    setZoom(Math.min(zoom * 1.2, 2));
  };

  const handleZoomOut = () => {
    setZoom(Math.max(zoom / 1.2, 0.2));
  };

  const handleReset = () => {
    setZoom(0.5);
    setPan({ x: 0, y: 0 });
  };

  return (
    <Card className="bg-card/80 backdrop-blur-sm border-aviation-blue/20 shadow-panel h-full">
      <CardContent className="p-4 h-full flex flex-col">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-aviation-blue font-semibold">World Map</h3>
          <div className="flex items-center gap-2">
            <Button variant="control" size="sm" onClick={handleZoomIn}>
              <ZoomIn className="w-4 h-4" />
            </Button>
            <Button variant="control" size="sm" onClick={handleZoomOut}>
              <ZoomOut className="w-4 h-4" />
            </Button>
            <Button variant="control" size="sm" onClick={handleReset}>
              <RotateCcw className="w-4 h-4" />
            </Button>
            <Badge variant="outline" className="ml-2">
              Zoom: {Math.round(zoom * 100)}%
            </Badge>
          </div>
        </div>

        <div
          ref={mapRef}
          className="flex-1 bg-gradient-to-br from-sky-100 to-blue-200 dark:from-slate-800 dark:to-slate-900 rounded-lg border border-aviation-blue/20 relative overflow-hidden cursor-grab active:cursor-grabbing"
          onMouseDown={handleMouseDown}
          onMouseMove={handleMouseMove}
          onMouseUp={handleMouseUp}
          onMouseLeave={handleMouseUp}
        >
          <div
            className="absolute inset-0"
            style={{
              transform: `scale(${zoom}) translate(${pan.x}px, ${pan.y}px)`,
              transformOrigin: '0 0',
              width: '10000px',
              height: '10000px',
            }}
          >
            {/* Grid Pattern */}
            <div className="absolute inset-0 opacity-20">
              {Array.from({ length: 100 }, (_, i) => (
                <div key={`h-${i}`} className="absolute w-full h-px bg-aviation-blue/30" style={{ top: `${i * 100}px` }} />
              ))}
              {Array.from({ length: 100 }, (_, i) => (
                <div key={`v-${i}`} className="absolute h-full w-px bg-aviation-blue/30" style={{ left: `${i * 100}px` }} />
              ))}
            </div>

            {/* Flight Paths */}
            {airplanes
              .filter(plane => plane.status === 'en-route' && plane.destination)
              .map(plane => {
                const destAirport = airports.find(a => a.code === plane.destination);
                if (!destAirport) return null;
                return (
                  <line
                    key={`path-${plane.id}`}
                    x1={plane.x}
                    y1={plane.y}
                    x2={destAirport.x}
                    y2={destAirport.y}
                    stroke="hsl(var(--aviation-amber))"
                    strokeWidth="2"
                    strokeDasharray="5,5"
                    className="opacity-60"
                  />
                );
              })}

            {/* Planned Paths Overlay */}
            {plannedPaths && plannedPaths.map((p, idx) => (
              <line
                key={`planned-${idx}`}
                x1={p.from.x}
                y1={p.from.y}
                x2={p.to.x}
                y2={p.to.y}
                stroke="hsl(var(--aviation-blue))"
                strokeWidth="2"
                strokeDasharray="2,4"
                className="opacity-70"
              />
            ))}

            {/* Airports */}
            {airports.map((airport) => (
              <div
                key={airport.id}
                className="absolute transform -translate-x-1/2 -translate-y-1/2 cursor-pointer group"
                style={{ left: `${airport.x}px`, top: `${airport.y}px` }}
                onClick={() => onAirportClick(airport)}
              >
                <div className="relative">
                  <div className={`w-6 h-6 rounded-full border-2 flex items-center justify-center transition-all group-hover:scale-110 ${
                    airport.hasOrders 
                      ? 'bg-aviation-amber border-aviation-amber/80 shadow-glow' 
                      : 'bg-aviation-blue border-aviation-blue/80'
                  }`}>
                    <MapPin className="w-3 h-3 text-white" />
                  </div>
                  {airport.hasOrders && (
                    <Badge 
                      variant="destructive" 
                      className="absolute -top-2 -right-2 text-xs w-5 h-5 p-0 rounded-full flex items-center justify-center"
                    >
                      {airport.orderCount}
                    </Badge>
                  )}
                  <div className="absolute top-7 left-1/2 transform -translate-x-1/2 whitespace-nowrap">
                    <div className="bg-black/80 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity">
                      {airport.code} - {airport.name}
                    </div>
                  </div>
                </div>
              </div>
            ))}

            {/* Airplanes */}
            {airplanes.map((airplane) => (
              <div
                key={airplane.id}
                className="absolute transform -translate-x-1/2 -translate-y-1/2 cursor-pointer group"
                style={{ left: `${airplane.x}px`, top: `${airplane.y}px` }}
                onClick={() => onAirplaneClick(airplane)}
              >
                <div className="relative">
                  <div className={`w-5 h-5 rounded-full border-2 flex items-center justify-center transition-all group-hover:scale-110 ${
                    airplane.status === 'en-route' 
                      ? 'bg-aviation-radar border-aviation-radar/80 animate-pulse' 
                      : airplane.status === 'loading'
                      ? 'bg-aviation-amber border-aviation-amber/80'
                      : 'bg-slate-500 border-slate-500/80'
                  }`}>
                    <Plane className="w-3 h-3 text-white" />
                  </div>
                  <div className="absolute top-6 left-1/2 transform -translate-x-1/2 whitespace-nowrap">
                    <div className="bg-black/80 text-white text-xs px-2 py-1 rounded opacity-0 group-hover:opacity-100 transition-opacity">
                      {airplane.id} - {airplane.model}
                      <br />
                      Status: {airplane.status}
                    </div>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
};
