"use client";

import React, { useMemo } from "react";
import { BarChart2 } from "lucide-react";
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
} from "recharts";
import { Card, CardHeader } from "@/components/common/Card";
import { generateChartData } from "@/lib/mock";

interface TooltipPayload {
  name: string;
  value: number;
  color: string;
}

interface CustomTooltipProps {
  active?: boolean;
  payload?: TooltipPayload[];
  label?: string;
}

const CustomTooltip = ({ active, payload, label }: CustomTooltipProps) => {
  if (!active || !payload?.length) return null;
  return (
    <div className="bg-vault-surface border border-vault-border rounded-lg px-3 py-2 text-xs shadow-xl">
      <p className="text-vault-muted mb-1 mono">{label}</p>
      {payload.map((p) => (
        <p key={p.name} className="mono" style={{ color: p.color }}>
          {p.name}: {p.value} SOL
        </p>
      ))}
    </div>
  );
};

export function ActivityChart() {
  const data = useMemo(() => generateChartData(), []);

  return (
    <Card>
      <CardHeader
        title="Trading Activity"
        icon={<BarChart2 size={16} />}
        subtitle="Last 24 hours"
        action={
          <div className="flex items-center gap-4 text-xs">
            <span className="flex items-center gap-1.5 text-vault-muted">
              <span className="w-2 h-2 rounded-full bg-sol-green inline-block" />
              Volume
            </span>
            <span className="flex items-center gap-1.5 text-vault-muted">
              <span className="w-2 h-2 rounded-full bg-sol-purple inline-block" />
              Cumulative
            </span>
          </div>
        }
      />

      <div className="h-48">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={data}
            margin={{ top: 5, right: 5, bottom: 0, left: -20 }}
          >
            <defs>
              <linearGradient id="colorVolume" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#14F195" stopOpacity={0.2} />
                <stop offset="95%" stopColor="#14F195" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="colorCumulative" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="#9945FF" stopOpacity={0.2} />
                <stop offset="95%" stopColor="#9945FF" stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="label"
              tick={{ fill: "#A0A0B0", fontSize: 10, fontFamily: "Space Mono" }}
              tickLine={false}
              axisLine={false}
              interval={3}
            />
            <YAxis
              tick={{ fill: "#A0A0B0", fontSize: 10, fontFamily: "Space Mono" }}
              tickLine={false}
              axisLine={false}
            />
            <Tooltip content={<CustomTooltip />} />
            <Area
              type="monotone"
              dataKey="volume"
              name="Volume"
              stroke="#14F195"
              strokeWidth={2}
              fill="url(#colorVolume)"
            />
            <Area
              type="monotone"
              dataKey="cumulative"
              name="Cumulative"
              stroke="#9945FF"
              strokeWidth={2}
              fill="url(#colorCumulative)"
              strokeDasharray="4 4"
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </Card>
  );
}
