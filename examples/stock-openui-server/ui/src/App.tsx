import { AiUi } from "@nube/ai-ui-react";

export default function App() {
  return (
    <AiUi
      agentName="ai-ui demo"
      conversationStarters={{
        variant: "short",
        options: [
          {
            displayText: "IoT dashboard",
            prompt:
              "Build me a dashboard for the chiller plant — KPI tiles for each chiller's kW, a 24h line chart of total load, and a table of recent alarms.",
          },
          {
            displayText: "Scope mock",
            prompt:
              "Mock a Secrets tab in settings: encrypted-at-rest API keys, listed by alias, with copy and delete buttons.",
          },
          {
            displayText: "Contact form",
            prompt:
              "Build a contact form with name, email, topic, and message fields.",
          },
          {
            displayText: "Data table",
            prompt:
              "Show a table of the top 5 programming languages by popularity.",
          },
        ],
      }}
    />
  );
}
