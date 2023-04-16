/** Update Markdown files in /bench */

import { BenchData, Markdown } from "./utils";

(async () => {
  const bench = await BenchData.open();

  await BenchData.forEachMarkdown((markdown) => {
    // Create table
    const table = Markdown.createTable("Instruction", "Compute Units", "+/-");

    const newComputeUnitsResult = bench.getUnreleased().computeUnits;
    const oldComputeUnitsResult = bench.getLastVersionResult().computeUnits;
    bench.compareComputeUnits(
      newComputeUnitsResult,
      oldComputeUnitsResult,
      (ixName, newComputeUnits, oldComputeUnits) => {
        const percentChange = (
          (newComputeUnits / oldComputeUnits - 1) *
          100
        ).toFixed(2);

        let changeText;
        if (isNaN(oldComputeUnits)) {
          changeText = "N/A";
        } else if (+percentChange > 0) {
          changeText = `🔴 **+${percentChange}%**`;
        } else {
          changeText = `🟢 **${percentChange}%**`;
        }

        table.insert(ixName, newComputeUnits.toString(), changeText);
      },
      (ixName, newComputeUnits) => {
        table.insert(ixName, newComputeUnits.toString(), "-");
      }
    );

    // Update unreleased section
    markdown.updateUnreleased(table);
  });
})();
